import * as pdfjsLib from 'pdfjs-dist';

// Setting up the worker
import pdfjsWorker from 'pdfjs-dist/build/pdf.worker.min.js?url';

pdfjsLib.GlobalWorkerOptions.workerSrc = pdfjsWorker;

export const loadPdfDocument = async (file: File) => {
    const arrayBuffer = await file.arrayBuffer();
    const loadingTask = pdfjsLib.getDocument(arrayBuffer);
    return loadingTask.promise;
};

export const renderPageToCanvas = async (
    pdfDoc: pdfjsLib.PDFDocumentProxy,
    pageNumber: number,
    canvas: HTMLCanvasElement
) => {
    const page = await pdfDoc.getPage(pageNumber);
    const viewport = page.getViewport({ scale: 1.5 }); // Adjust scale as needed

    canvas.height = viewport.height;
    canvas.width = viewport.width;

    const renderContext = {
        canvasContext: canvas.getContext('2d')!,
        viewport: viewport,
    };

    await page.render(renderContext).promise;
    return viewport;
};
