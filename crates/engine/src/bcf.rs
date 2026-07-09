use crate::clash::{ClashCheckResult, ClashResult, ClashSeverity};
use crate::project::Project;
use anyhow::Result;
use chrono::Utc;
use std::io::{Cursor, Write};
use uuid::Uuid;
use zip::write::{SimpleFileOptions, ZipWriter};
use zip::CompressionMethod;

const BCF_VERSION_XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<Version VersionId="2.1" xmlns="http://www.buildingsmart-tech.org/specifications/bcf-xml">
  <DetailedVersion>2.1</DetailedVersion>
</Version>
"#;

/// Builds a BCF 2.1 (.bcfzip) archive: one Topic per detected Hard clash.
/// Skipped pairs (missing geometry etc.) aren't issues to raise — they're
/// not included. Object names come straight from IFC file content, which
/// is untrusted input, so every string that lands in the XML is escaped.
pub fn export_clashes_to_bcf(project: &Project, results: &[ClashCheckResult]) -> Result<Vec<u8>> {
    let mut buf = Cursor::new(Vec::new());
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

    {
        let mut zip = ZipWriter::new(&mut buf);

        zip.start_file("bcf.version", options)?;
        zip.write_all(BCF_VERSION_XML.as_bytes())?;

        for result in results {
            if let ClashCheckResult::Clash(clash) = result {
                let topic_guid = Uuid::new_v4();
                let markup = build_markup_xml(project, clash, topic_guid);
                zip.start_file(format!("{topic_guid}/markup.bcf"), options)?;
                zip.write_all(markup.as_bytes())?;
            }
        }

        zip.finish()?;
    }

    Ok(buf.into_inner())
}

fn build_markup_xml(project: &Project, clash: &ClashResult, topic_guid: Uuid) -> String {
    let name_a = object_name(project, clash.object_a);
    let name_b = object_name(project, clash.object_b);
    let title = format!("Clash: {name_a} \u{d7} {name_b}");
    let description = format!(
        "Hard clash detected between \"{name_a}\" and \"{name_b}\". Overlap volume: {:.4} m\u{b3}.",
        clash.overlap_volume
    );
    let priority = severity_to_priority(clash.severity);
    let created = Utc::now().to_rfc3339();

    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<Markup xmlns="http://www.buildingsmart-tech.org/specifications/bcf-xml">
  <Topic Guid="{topic_guid}" TopicType="Clash" TopicStatus="Open">
    <Title>{title}</Title>
    <Priority>{priority}</Priority>
    <Description>{description}</Description>
    <CreationDate>{created}</CreationDate>
    <CreationAuthor>Open Construction Modeler</CreationAuthor>
  </Topic>
</Markup>
"#,
        title = xml_escape(&title),
        description = xml_escape(&description),
    )
}

fn object_name(project: &Project, id: Uuid) -> String {
    project
        .get_object(&id)
        .map(|o| o.name.clone())
        .unwrap_or_else(|| id.to_string())
}

fn severity_to_priority(severity: ClashSeverity) -> &'static str {
    match severity {
        ClashSeverity::Critical => "High",
        ClashSeverity::Major => "Normal",
        ClashSeverity::Minor => "Low",
    }
}

fn xml_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clash::{ClashType, MissingGeometryReason, SkippedResult};
    use crate::metadata::{LodLevel, Trade};
    use crate::object::ConstructionObject;
    use std::io::Read;
    use zip::ZipArchive;

    fn make_project_with_objects(names: &[&str]) -> (Project, Vec<Uuid>) {
        let mut project = Project::new("Test Project".to_string());
        let mut ids = Vec::new();
        for name in names {
            let obj = ConstructionObject::new(
                name.to_string(),
                Trade::Structural,
                None,
                LodLevel::Lod200,
                String::new(),
                String::new(),
            );
            ids.push(obj.id);
            project.add_object(obj);
        }
        (project, ids)
    }

    fn make_clash(object_a: Uuid, object_b: Uuid, severity: ClashSeverity) -> ClashCheckResult {
        ClashCheckResult::Clash(ClashResult {
            object_a,
            object_b,
            overlap: [1.0, 1.0, 1.0],
            position: [0.0, 0.0, 0.0],
            overlap_volume: 1.0,
            clash_type: ClashType::Hard,
            severity,
        })
    }

    #[test]
    fn test_export_produces_valid_zip_with_expected_entries() {
        let (project, ids) = make_project_with_objects(&["Wall A", "Duct B"]);
        let results = vec![make_clash(ids[0], ids[1], ClashSeverity::Critical)];

        let bytes = export_clashes_to_bcf(&project, &results).unwrap();
        let mut archive = ZipArchive::new(Cursor::new(bytes)).unwrap();

        // bcf.version + one topic folder's markup.bcf
        assert_eq!(archive.len(), 2);

        let mut version_file = archive.by_name("bcf.version").unwrap();
        let mut version_contents = String::new();
        version_file.read_to_string(&mut version_contents).unwrap();
        assert!(version_contents.contains("VersionId=\"2.1\""));
    }

    #[test]
    fn test_markup_contains_object_names_and_severity() {
        let (project, ids) = make_project_with_objects(&["Wall A", "Duct B"]);
        let results = vec![make_clash(ids[0], ids[1], ClashSeverity::Critical)];

        let bytes = export_clashes_to_bcf(&project, &results).unwrap();
        let mut archive = ZipArchive::new(Cursor::new(bytes)).unwrap();

        let markup_name = (0..archive.len())
            .map(|i| archive.by_index(i).unwrap().name().to_string())
            .find(|n| n.ends_with("markup.bcf"))
            .expect("a markup.bcf entry must exist");

        let mut markup_file = archive.by_name(&markup_name).unwrap();
        let mut contents = String::new();
        markup_file.read_to_string(&mut contents).unwrap();

        assert!(contents.contains("Wall A"));
        assert!(contents.contains("Duct B"));
        assert!(contents.contains("<Priority>High</Priority>"));
        assert!(contents.contains("TopicStatus=\"Open\""));
    }

    #[test]
    fn test_object_names_are_xml_escaped() {
        // A malicious/malformed IFC name must not break the XML or inject markup
        let (project, ids) = make_project_with_objects(&[
            "Wall <script>alert(1)</script> & \"quoted\"",
            "Duct B",
        ]);
        let results = vec![make_clash(ids[0], ids[1], ClashSeverity::Minor)];

        let bytes = export_clashes_to_bcf(&project, &results).unwrap();
        let mut archive = ZipArchive::new(Cursor::new(bytes)).unwrap();
        let markup_name = (0..archive.len())
            .map(|i| archive.by_index(i).unwrap().name().to_string())
            .find(|n| n.ends_with("markup.bcf"))
            .unwrap();
        let mut markup_file = archive.by_name(&markup_name).unwrap();
        let mut contents = String::new();
        markup_file.read_to_string(&mut contents).unwrap();

        assert!(!contents.contains("<script>"));
        assert!(contents.contains("&lt;script&gt;"));
        assert!(contents.contains("&amp;"));
        assert!(contents.contains("&quot;quoted&quot;"));
    }

    #[test]
    fn test_skipped_results_produce_no_topics() {
        let (project, ids) = make_project_with_objects(&["Wall A", "Duct B"]);
        let results = vec![ClashCheckResult::Skipped(SkippedResult {
            object: ids[0],
            reason: MissingGeometryReason::NoPosition,
        })];

        let bytes = export_clashes_to_bcf(&project, &results).unwrap();
        let archive = ZipArchive::new(Cursor::new(bytes)).unwrap();
        assert_eq!(archive.len(), 1); // bcf.version only
    }

    #[test]
    fn test_empty_results_still_produces_valid_archive() {
        let (project, _ids) = make_project_with_objects(&[]);
        let bytes = export_clashes_to_bcf(&project, &[]).unwrap();
        let archive = ZipArchive::new(Cursor::new(bytes)).unwrap();
        assert_eq!(archive.len(), 1);
    }

    #[test]
    fn test_missing_object_falls_back_to_uuid_instead_of_panicking() {
        let project = Project::new("Empty".to_string());
        let orphan_a = Uuid::new_v4();
        let orphan_b = Uuid::new_v4();
        let results = vec![make_clash(orphan_a, orphan_b, ClashSeverity::Minor)];

        let bytes = export_clashes_to_bcf(&project, &results).unwrap();
        let mut archive = ZipArchive::new(Cursor::new(bytes)).unwrap();
        let markup_name = (0..archive.len())
            .map(|i| archive.by_index(i).unwrap().name().to_string())
            .find(|n| n.ends_with("markup.bcf"))
            .unwrap();
        let mut markup_file = archive.by_name(&markup_name).unwrap();
        let mut contents = String::new();
        markup_file.read_to_string(&mut contents).unwrap();
        assert!(contents.contains(&orphan_a.to_string()));
    }
}
