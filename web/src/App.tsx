import { useState, useEffect } from 'react';
import * as pdfjsLib from 'pdfjs-dist';
import { DropZone } from './components/DropZone';
import { PdfCanvas } from './components/PdfCanvas';
import { Eraser, Move, Undo, Redo, Trash2, ArrowLeft, ArrowRight, Save, Square, Layers, HelpCircle, X } from 'lucide-react';
import { save } from '@tauri-apps/plugin-dialog';
import { writeFile } from '@tauri-apps/plugin-fs';
import { translations, Language } from './locales';

// Initialize PDF.js Worker
pdfjsLib.GlobalWorkerOptions.workerSrc = new URL(
    'pdfjs-dist/build/pdf.worker.min.js',
    import.meta.url
).toString();

// Interface matching Rust
interface MaskRect {
    page: number;
    x: number;
    y: number;
    width: number;
    height: number;
    color: 'black' | 'white';
}

function App() {
    const [file, setFile] = useState<File | null>(null);
    const [pdfDoc, setPdfDoc] = useState<pdfjsLib.PDFDocumentProxy | null>(null);
    const [currentPage, setCurrentPage] = useState(1);
    const [totalPages, setTotalPages] = useState(0);
    const [masks, setMasks] = useState<MaskRect[]>([]);
    const [isSanitizing, setIsSanitizing] = useState(false);
    const [activeTool, setActiveTool] = useState<'black' | 'white' | 'move'>('black');
    const [selectedMaskIndex, setSelectedMaskIndex] = useState<number | null>(null);
    const [history, setHistory] = useState<MaskRect[][]>([]);
    const [redoStack, setRedoStack] = useState<MaskRect[][]>([]);
    const [showHelp, setShowHelp] = useState(false);
    const [lang, setLang] = useState<Language>('ja');

    const t = translations[lang];

    useEffect(() => {
        if (file) {
            const load = async (password?: string) => {
                try {
                    const arrayBuffer = await file.arrayBuffer();
                    // Initialize PDF.js
                    const loadingTask = pdfjsLib.getDocument({
                        data: arrayBuffer,
                        password: password
                    });

                    loadingTask.onPassword = (updatePassword: (pw: string) => void, reason: number) => {
                        const pw = prompt(reason === 1 ? t.passwordPrompt : t.passwordIncorrect);
                        if (pw !== null) {
                            updatePassword(pw);
                        } else {
                            // User cancelled
                            setFile(null);
                        }
                    };

                    const doc = await loadingTask.promise;
                    setPdfDoc(doc);
                    setTotalPages(doc.numPages);
                    setCurrentPage(1);
                    setMasks([]);
                } catch (error: any) {
                    if (error.name === 'PasswordException') {
                        // This case is usually handled by onPassword, but if it fails/falls through:
                        const pw = prompt(t.passwordPrompt);
                        if (pw !== null) {
                            load(pw);
                        } else {
                            setFile(null);
                        }
                    } else {
                        console.error("Failed to load PDF:", error);
                        alert(`${t.loadError}\n${error}`);
                        setFile(null); // Reset to allow retry
                    }
                }
            };
            load();
        }
    }, [file, lang]);

    const saveHistory = (newMasks: MaskRect[]) => {
        setHistory(prev => [...prev, masks].slice(-10));
        setRedoStack([]);
        setMasks(newMasks);
    };

    const handleUndo = () => {
        if (history.length > 0) {
            const prevState = history[history.length - 1];
            setRedoStack(prev => [...prev, masks]);
            setHistory(prev => prev.slice(0, -1));
            setMasks(prevState);
            setSelectedMaskIndex(null);
        }
    };

    const handleRedo = () => {
        if (redoStack.length > 0) {
            const nextState = redoStack[redoStack.length - 1];
            setHistory(prev => [...prev, masks]);
            setRedoStack(prev => prev.slice(0, -1));
            setMasks(nextState);
            setSelectedMaskIndex(null);
        }
    };

    const handleDeleteSelected = () => {
        if (selectedMaskIndex !== null) {
            const newMasks = [...masks];
            newMasks.splice(selectedMaskIndex, 1);
            saveHistory(newMasks);
            setSelectedMaskIndex(null);
        }
    };

    const handleApplyToAllPages = () => {
        if (selectedMaskIndex !== null) {
            const targetMask = masks[selectedMaskIndex];
            const newMasks = [...masks];

            for (let p = 1; p <= totalPages; p++) {
                if (p === targetMask.page) continue; // Skip current page

                // Add copy to this page
                newMasks.push({
                    ...targetMask,
                    page: p
                });
            }
            saveHistory(newMasks);
            alert(t.applyAllAlert.replace('{total}', String(totalPages)));
        }
    };

    useEffect(() => {
        const handleKeyDown = (e: KeyboardEvent) => {
            if ((e.metaKey || e.ctrlKey) && e.key === 'z') {
                if (e.shiftKey) {
                    handleRedo();
                } else {
                    handleUndo();
                }
            } else if ((e.key === 'Delete' || e.key === 'Backspace') && selectedMaskIndex !== null) {
                handleDeleteSelected();
            }
        };
        window.addEventListener('keydown', handleKeyDown);
        return () => window.removeEventListener('keydown', handleKeyDown);
    }, [masks, history, redoStack, selectedMaskIndex]);

    const handleSanitize = async () => {
        if (!file) return;
        setIsSanitizing(true);

        try {
            // Dynamic import WASM to avoid load errors if it's missing initially
            const wasm = await import('../../wasm/pkg/masking_tool_wasm');
            await wasm.default(); // init if needed, though vite plugin handles it usually

            const arrayBuffer = await file.arrayBuffer();
            const pdfBytes = new Uint8Array(arrayBuffer);

            // Call WASM
            const resultBytes = wasm.sanitize_pdf(pdfBytes, masks);

            // Native Save
            let path: string | null = null;
            try {
                // Generate filename with _masked suffix
                let baseName = file.name;
                if (baseName.toLowerCase().endsWith('.pdf')) {
                    baseName = baseName.substring(0, baseName.length - 4);
                }
                const suggestedName = `${baseName}_masked.pdf`;

                path = await save({
                    defaultPath: suggestedName,
                    filters: [{
                        name: 'PDF Files',
                        extensions: ['pdf']
                    }]
                });

                if (path) {
                    await writeFile(path, resultBytes);
                    if (confirm(`${t.saveTitle}\n${t.saveMessage.replace('{path}', path)}`)) {
                        setFile(null);
                    }
                }
            } catch (err) {
                console.error("Native save failed, fallback to download", err);
                const errorMessage = err instanceof Error ? err.message : String(err);
                alert(`${t.saveError.replace('{error}', errorMessage)}\n\npath: ${path !== null ? path : 'not selected'}`);
                // Fallback (though this causes error -999 in some webviews, we keep it just in case)
                const buffer = resultBytes.buffer instanceof ArrayBuffer ? resultBytes.buffer : new Uint8Array(resultBytes).buffer;
                const blob = new Blob([buffer], { type: 'application/pdf' });
                const url = URL.createObjectURL(blob);
                const a = document.createElement('a');
                a.href = url;
                a.download = `${file.name.replace(/\.pdf$/i, '')}_masked.pdf`;
                document.body.appendChild(a);
                a.click();
                document.body.removeChild(a);
                URL.revokeObjectURL(url);
            }

        } catch (e) {
            console.error("Sanitization failed:", e);
            alert("Error (WASM not loaded?)\n" + e);
        } finally {
            setIsSanitizing(false);
        }
    };

    return (
        <div className="min-h-screen bg-gray-100 flex flex-col items-center py-10 relative">
            <header className="mb-8 text-center relative w-full max-w-5xl px-4 md:px-0">
                <h1 className="text-3xl font-bold text-gray-900">{t.title}</h1>
                <p className="text-gray-600 mt-2">{t.subtitle}</p>

                <div className="absolute right-0 top-1/2 -translate-y-1/2 flex items-center gap-2">
                    {/* Language Switcher */}
                    <div className="flex bg-white rounded-full shadow border border-gray-200 p-1 mr-2">
                        <button onClick={() => setLang('zh')} className={`px-2 py-1 text-xs rounded-full font-bold transition ${lang === 'zh' ? 'bg-blue-600 text-white' : 'text-gray-500 hover:bg-gray-100'}`}>中</button>
                        <button onClick={() => setLang('ja')} className={`px-2 py-1 text-xs rounded-full font-bold transition ${lang === 'ja' ? 'bg-blue-600 text-white' : 'text-gray-500 hover:bg-gray-100'}`}>日</button>
                        <button onClick={() => setLang('en')} className={`px-2 py-1 text-xs rounded-full font-bold transition ${lang === 'en' ? 'bg-blue-600 text-white' : 'text-gray-500 hover:bg-gray-100'}`}>EN</button>
                    </div>

                    <button
                        onClick={() => setShowHelp(true)}
                        className="flex items-center gap-2 text-blue-600 hover:text-blue-800 bg-white px-3 py-1 rounded-full shadow hover:shadow-md transition"
                    >
                        <HelpCircle className="w-5 h-5" />
                        <span className="font-bold hidden md:inline">{t.helpButton}</span>
                    </button>
                </div>
            </header>

            {showHelp && (
                <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4" onClick={() => setShowHelp(false)}>
                    <div className="bg-white rounded-xl shadow-2xl max-w-2xl w-full max-h-[80vh] overflow-auto relative" onClick={e => e.stopPropagation()}>
                        <button
                            onClick={() => setShowHelp(false)}
                            className="absolute right-4 top-4 text-gray-400 hover:text-gray-600 p-1 rounded-full hover:bg-gray-100"
                        >
                            <X className="w-6 h-6" />
                        </button>

                        <div className="p-8">
                            <h2 className="text-2xl font-bold mb-6 border-b pb-2 flex items-center gap-2">
                                <HelpCircle className="w-6 h-6 text-blue-600" />
                                {t.helpTitle}
                            </h2>

                            <div className="space-y-6 text-gray-700 leading-relaxed">
                                <section>
                                    <h3 className="text-lg font-bold text-gray-900 mb-2">{t.step1Title}</h3>
                                    <p>{t.step1Desc}</p>
                                </section>

                                <section>
                                    <h3 className="text-lg font-bold text-gray-900 mb-2">{t.step2Title}</h3>
                                    <ul className="list-disc list-inside ml-2 space-y-1">
                                        <li>{t.step2Desc1}</li>
                                        <li>{t.step2Desc2}</li>
                                    </ul>
                                </section>

                                <section>
                                    <h3 className="text-lg font-bold text-gray-900 mb-2">{t.step3Title}</h3>
                                    <ul className="list-disc list-inside ml-2 space-y-1">
                                        <li>{t.step3Desc1}</li>
                                        <li>{t.step3Desc2}</li>
                                        <li>{t.step3Desc3}</li>
                                        <li>{t.step3Desc4}</li>
                                    </ul>
                                </section>

                                <section>
                                    <h3 className="text-lg font-bold text-gray-900 mb-2">{t.step4Title}</h3>
                                    <p>{t.step4Desc}</p>
                                </section>

                                <div className="bg-blue-50 p-4 rounded text-sm text-blue-800 border border-blue-100 mt-4">
                                    <strong>{t.securityNoteTitle}</strong><br />
                                    {t.securityNoteDesc}
                                </div>
                                <div className="text-xs text-gray-400 mt-2 text-center">
                                    {t.openSourceNote}
                                </div>
                            </div>

                            <div className="mt-8 text-center">
                                <button
                                    onClick={() => setShowHelp(false)}
                                    className="bg-blue-600 text-white px-6 py-2 rounded-full hover:bg-blue-700 font-bold transition"
                                >
                                    {t.helpClose}
                                </button>
                            </div>
                        </div>
                    </div>
                </div>
            )}

            <div className="w-full max-w-5xl bg-white p-6 rounded-lg shadow-md flex flex-col">
                {!file && (
                    <DropZone
                        onFileSelect={setFile}
                        text={t.dropZoneText}
                        subText={t.dropZoneSub}
                    />
                )}

                {file && pdfDoc && (
                    <div className="flex flex-col items-center w-full">
                        <div className="w-full flex justify-start mb-2">
                            <button
                                onClick={() => {
                                    if (confirm(t.confirmReset)) {
                                        setFile(null);
                                    }
                                }}
                                className="flex items-center gap-2 text-gray-500 hover:text-gray-800 transition px-2 py-1 rounded hover:bg-gray-100"
                            >
                                <ArrowLeft className="w-4 h-4" />
                                <span className="font-bold">{t.resetFile}</span>
                            </button>
                        </div>

                        {/* Toolbar */}
                        <div className="w-full flex items-center justify-between mb-4 bg-gray-50 p-2 rounded border border-gray-200 sticky top-0 z-10 shadow-sm flex-wrap gap-2">
                            <div className="flex gap-2">
                                <button
                                    onClick={() => setActiveTool('black')}
                                    className={`px-4 py-2 rounded flex items-center gap-2 border transition ${activeTool === 'black' ? 'bg-gray-800 text-white border-gray-900 shadow-md' : 'bg-white text-gray-700 border-gray-200 hover:bg-gray-100'}`}
                                >
                                    <Square className="w-4 h-4 fill-current" />
                                    <span className="font-medium hidden sm:inline">{t.toolBlack}</span>
                                </button>
                                <button
                                    onClick={() => setActiveTool('white')}
                                    className={`px-4 py-2 rounded flex items-center gap-2 border transition ${activeTool === 'white' ? 'bg-gray-100 text-gray-900 border-gray-300 shadow-md' : 'bg-white text-gray-700 border-gray-200 hover:bg-gray-50'}`}
                                >
                                    <Eraser className="w-4 h-4" />
                                    <span className="font-medium hidden sm:inline">{t.toolWhite}</span>
                                </button>
                                <button
                                    onClick={() => setActiveTool('move')}
                                    className={`px-4 py-2 rounded flex items-center gap-2 border transition ${activeTool === 'move' ? 'bg-blue-600 text-white border-blue-700 shadow-md' : 'bg-white text-gray-700 border-gray-200 hover:bg-gray-100'}`}
                                >
                                    <Move className="w-4 h-4" />
                                    <span className="font-medium hidden sm:inline">{t.toolMove}</span>
                                </button>
                            </div>

                            <div className="h-6 w-[1px] bg-gray-300 mx-2 hidden sm:block" />

                            <div className="flex gap-2">
                                <button
                                    onClick={handleUndo}
                                    disabled={history.length === 0}
                                    className="p-2 rounded bg-white border border-gray-200 hover:bg-gray-100 disabled:opacity-30 disabled:hover:bg-white transition text-gray-700"
                                    title={t.actionUndo}
                                >
                                    <Undo className="w-5 h-5" />
                                </button>
                                <button
                                    onClick={handleRedo}
                                    disabled={redoStack.length === 0}
                                    className="p-2 rounded bg-white border border-gray-200 hover:bg-gray-100 disabled:opacity-30 disabled:hover:bg-white transition text-gray-700"
                                    title={t.actionRedo}
                                >
                                    <Redo className="w-5 h-5" />
                                </button>
                                <button
                                    onClick={handleApplyToAllPages}
                                    disabled={selectedMaskIndex === null}
                                    className={`px-4 py-2 rounded flex items-center gap-2 border transition ${selectedMaskIndex !== null ? 'bg-indigo-50 text-indigo-600 border-indigo-200 hover:bg-indigo-100' : 'bg-white text-gray-300 border-gray-200'}`}
                                    title={t.actionApplyAll}
                                >
                                    <Layers className="w-4 h-4" />
                                    <span className="hidden sm:inline">{t.actionApplyAll}</span>
                                </button>
                                <button
                                    onClick={handleDeleteSelected}
                                    disabled={selectedMaskIndex === null}
                                    className={`px-4 py-2 rounded flex items-center gap-2 border transition ${selectedMaskIndex !== null ? 'bg-red-50 text-red-600 border-red-200 hover:bg-red-100' : 'bg-white text-gray-300 border-gray-200'}`}
                                    title={t.actionDelete}
                                >
                                    <Trash2 className="w-4 h-4" />
                                    <span className="hidden sm:inline">{t.actionDeleteKey}</span>
                                </button>
                            </div>

                            <div className="flex bg-white rounded-lg p-1 border border-gray-200 shadow-sm ml-auto mt-2 sm:mt-0">
                                <button
                                    disabled={currentPage <= 1}
                                    onClick={() => setCurrentPage(p => p - 1)}
                                    className="p-1 rounded hover:bg-gray-100 disabled:opacity-30 transition text-gray-600"
                                >
                                    <ArrowLeft className="w-5 h-5" />
                                </button>
                                <span className="px-3 py-1 font-mono min-w-[60px] text-center text-sm flex items-center justify-center text-gray-600 border-x border-gray-100 mx-1">
                                    {currentPage} / {totalPages}
                                </span>
                                <button
                                    disabled={currentPage >= totalPages}
                                    onClick={() => setCurrentPage(p => p + 1)}
                                    className="p-1 rounded hover:bg-gray-100 disabled:opacity-30 transition text-gray-600"
                                >
                                    <ArrowRight className="w-5 h-5" />
                                </button>
                            </div>
                        </div>

                        <div className="overflow-auto max-h-[70vh] border rounded bg-gray-50 relative w-full flex justify-center p-4">
                            <PdfCanvas
                                pdfDoc={pdfDoc}
                                pageNumber={currentPage}
                                masks={masks}
                                onMasksChange={saveHistory}
                                activeTool={activeTool}
                                selectedMaskIndex={selectedMaskIndex}
                                onSelectionChange={setSelectedMaskIndex}
                            />
                        </div>

                        <div className="mt-8 flex flex-col items-center gap-4 w-full">
                            <button
                                onClick={handleSanitize}
                                disabled={isSanitizing}
                                className="bg-blue-600 text-white px-8 py-3 rounded-full hover:bg-blue-700 disabled:opacity-50 font-bold text-lg shadow-lg transition-transform hover:scale-105 active:scale-95 border-2 border-blue-700 flex items-center gap-3"
                            >
                                <Save className="w-5 h-5" />
                                {isSanitizing ? t.processing : t.saveButton}
                            </button>

                            <div className="text-xs text-gray-400 flex gap-4">
                                <span>{t.statusMasks.replace('{count}', String(masks.length))}</span>
                                <span>{t.page}: {currentPage} / {totalPages}</span>
                            </div>
                        </div>
                    </div>
                )}
            </div>
        </div>
    );
}

export default App;
