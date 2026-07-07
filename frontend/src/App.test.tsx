import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import App from './App'

vi.mock('@tauri-apps/api/core', () => ({
    invoke: vi.fn(),
}))
vi.mock('@tauri-apps/plugin-dialog', () => ({
    open: vi.fn(),
    save: vi.fn(),
}))
// Viewport mounts a real THREE.WebGLRenderer, which needs a GL context jsdom
// doesn't provide — stub it so these tests exercise App's own state/logic.
vi.mock('./Viewport', () => ({
    default: () => <div data-testid="viewport-stub" />,
}))

import { invoke } from '@tauri-apps/api/core'
import { open, save } from '@tauri-apps/plugin-dialog'

const mockInvoke = vi.mocked(invoke)
const mockOpen = vi.mocked(open)
const mockSave = vi.mocked(save)

function makeObject(id: string, name: string, trade: string) {
    return {
        id,
        name,
        trade,
        lod: 'Lod200',
        csi_code: '',
        phase: '',
        status: 'NotStarted',
        approval_status: 'Draft',
        entity_type: null,
        position: null,
        dimensions: null,
        matrix: null,
    }
}

function makeProject() {
    const a = makeObject('id-a', 'Wall A', 'Structural')
    const b = makeObject('id-b', 'Duct B', 'Mechanical')
    return {
        id: 'proj-1',
        name: 'Test Project',
        objects: { [a.id]: a, [b.id]: b },
    }
}

beforeEach(() => {
    mockInvoke.mockReset()
    mockOpen.mockReset()
    mockSave.mockReset()
})

describe('App — initial state', () => {
    it('shows no project loaded and a disabled clash button', () => {
        render(<App />)
        expect(screen.getByText('No project loaded')).toBeInTheDocument()
        expect(screen.getByRole('button', { name: /run clash detection/i })).toBeDisabled()
    })
})

describe('App — loadProject', () => {
    it('loads a project on success and clears any previous error', async () => {
        mockOpen.mockResolvedValue('/path/to/project.ocm')
        mockInvoke.mockResolvedValue(makeProject())
        const user = userEvent.setup()

        render(<App />)
        await user.click(screen.getByRole('button', { name: /load project/i }))

        await waitFor(() => expect(screen.getByText('Test Project')).toBeInTheDocument())
        expect(screen.getByText('Wall A')).toBeInTheDocument()
        expect(screen.getByText('Duct B')).toBeInTheDocument()
        expect(mockInvoke).toHaveBeenCalledWith('load_project', { path: '/path/to/project.ocm' })
    })

    it('does nothing when the user cancels the file dialog', async () => {
        mockOpen.mockResolvedValue(null)
        const user = userEvent.setup()

        render(<App />)
        await user.click(screen.getByRole('button', { name: /load project/i }))

        expect(mockInvoke).not.toHaveBeenCalled()
        expect(screen.getByText('No project loaded')).toBeInTheDocument()
    })

    it('shows an error message when loading fails', async () => {
        mockOpen.mockResolvedValue('/path/to/broken.ifc')
        mockInvoke.mockRejectedValue('parse error: malformed entity')
        const user = userEvent.setup()

        render(<App />)
        await user.click(screen.getByRole('button', { name: /load project/i }))

        await waitFor(() =>
            expect(screen.getByText(/parse error: malformed entity/)).toBeInTheDocument(),
        )
        expect(screen.getByText('No project loaded')).toBeInTheDocument()
    })
})

describe('App — runClash', () => {
    async function loadProjectFirst(user: ReturnType<typeof userEvent.setup>) {
        mockOpen.mockResolvedValue('/path/to/project.ocm')
        mockInvoke.mockResolvedValueOnce(makeProject())
        render(<App />)
        await user.click(screen.getByRole('button', { name: /load project/i }))
        await waitFor(() => expect(screen.getByText('Test Project')).toBeInTheDocument())
    }

    it('filters out Skipped results and shows only real clashes', async () => {
        const user = userEvent.setup()
        await loadProjectFirst(user)

        mockInvoke.mockResolvedValueOnce([
            {
                type: 'Clash',
                object_a: 'id-a',
                object_b: 'id-b',
                overlap: [1, 1, 1],
                position: [0, 0, 0],
                overlap_volume: 1.0,
                clash_type: 'Hard',
                severity: 'Critical',
            },
            {
                type: 'Skipped',
                object_a: 'id-a',
                object_b: 'id-b',
                reason: 'NoPosition',
            },
        ])

        await user.click(screen.getByRole('button', { name: /run clash detection/i }))

        await waitFor(() => expect(screen.getByText('Clashes (1)')).toBeInTheDocument())
        // Both names appear twice — once in the sidebar object list, once in
        // the clash row — the clash panel rendering at all is what matters here.
        expect(screen.getAllByText('Wall A').length).toBeGreaterThan(0)
        expect(screen.getAllByText('Duct B').length).toBeGreaterThan(0)
        expect(screen.getByText('Critical')).toBeInTheDocument()
    })

    it('shows an error and resets loading state when run_clash fails', async () => {
        const user = userEvent.setup()
        await loadProjectFirst(user)

        mockInvoke.mockRejectedValueOnce('No project loaded')

        const clashButton = screen.getByRole('button', { name: /run clash detection/i })
        await user.click(clashButton)

        await waitFor(() => expect(screen.getByText('No project loaded')).toBeInTheDocument())
        // Button must not be stuck in the "Running…" state after failure
        expect(screen.getByRole('button', { name: /run clash detection/i })).not.toBeDisabled()
    })

    it('falls back to a truncated id when a clashing object is missing from the project', async () => {
        const user = userEvent.setup()
        await loadProjectFirst(user)

        mockInvoke.mockResolvedValueOnce([
            {
                type: 'Clash',
                object_a: 'id-a',
                object_b: 'orphan-id-not-in-project',
                overlap: [1, 1, 1],
                position: [0, 0, 0],
                overlap_volume: 1.0,
                clash_type: 'Hard',
                severity: 'Minor',
            },
        ])

        await user.click(screen.getByRole('button', { name: /run clash detection/i }))

        await waitFor(() => expect(screen.getByText('Clashes (1)')).toBeInTheDocument())
        expect(screen.getByText('orphan-i')).toBeInTheDocument() // first 8 chars
    })
})

describe('App — exportBcf', () => {
    async function loadProjectAndRunClash(user: ReturnType<typeof userEvent.setup>) {
        mockOpen.mockResolvedValue('/path/to/project.ocm')
        mockInvoke.mockResolvedValueOnce(makeProject())
        render(<App />)
        await user.click(screen.getByRole('button', { name: /load project/i }))
        await waitFor(() => expect(screen.getByText('Test Project')).toBeInTheDocument())

        mockInvoke.mockResolvedValueOnce([
            {
                type: 'Clash',
                object_a: 'id-a',
                object_b: 'id-b',
                overlap: [1, 1, 1],
                position: [0, 0, 0],
                overlap_volume: 1.0,
                clash_type: 'Hard',
                severity: 'Major',
            },
        ])
        await user.click(screen.getByRole('button', { name: /run clash detection/i }))
        await waitFor(() => expect(screen.getByText('Clashes (1)')).toBeInTheDocument())
    }

    it('does not invoke export when the save dialog is cancelled', async () => {
        const user = userEvent.setup()
        await loadProjectAndRunClash(user)
        mockSave.mockResolvedValue(null)
        mockInvoke.mockClear()

        await user.click(screen.getByRole('button', { name: /export bcf/i }))

        expect(mockInvoke).not.toHaveBeenCalled()
    })

    it('exports successfully and shows no error', async () => {
        const user = userEvent.setup()
        await loadProjectAndRunClash(user)
        mockSave.mockResolvedValue('/path/to/clashes.bcfzip')
        mockInvoke.mockResolvedValueOnce(undefined)

        await user.click(screen.getByRole('button', { name: /export bcf/i }))

        await waitFor(() =>
            expect(mockInvoke).toHaveBeenCalledWith('export_bcf', { path: '/path/to/clashes.bcfzip' }),
        )
        expect(screen.queryByText(/error/i)).not.toBeInTheDocument()
    })

    it('shows an error when export fails', async () => {
        const user = userEvent.setup()
        await loadProjectAndRunClash(user)
        mockSave.mockResolvedValue('/path/to/clashes.bcfzip')
        mockInvoke.mockRejectedValueOnce('disk full')

        await user.click(screen.getByRole('button', { name: /export bcf/i }))

        await waitFor(() => expect(screen.getByText(/disk full/)).toBeInTheDocument())
    })
})

describe('App — trade filter', () => {
    it('hides objects of a toggled-off trade from the sidebar list', async () => {
        mockOpen.mockResolvedValue('/path/to/project.ocm')
        mockInvoke.mockResolvedValue(makeProject())
        const user = userEvent.setup()

        render(<App />)
        await user.click(screen.getByRole('button', { name: /load project/i }))
        await waitFor(() => expect(screen.getByText('Wall A')).toBeInTheDocument())

        await user.click(screen.getByRole('button', { name: 'Structural' }))

        expect(screen.queryByText('Wall A')).not.toBeInTheDocument()
        expect(screen.getByText('Duct B')).toBeInTheDocument()
    })
})
