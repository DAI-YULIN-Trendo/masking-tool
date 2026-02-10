import React, { useCallback } from 'react';
import { clsx } from 'clsx';

interface DropZoneProps {
    onFileSelect: (file: File) => void;
    text?: string;
    subText?: string;
}

export const DropZone: React.FC<DropZoneProps> = ({
    onFileSelect,
    text = "ここにPDFをドラッグ＆ドロップ、またはクリックして選択",
    subText
}) => {
    const handleDrop = useCallback(
        (e: React.DragEvent<HTMLDivElement>) => {
            e.preventDefault();
            e.stopPropagation();
            const files = e.dataTransfer.files;
            if (files && files.length > 0) {
                if (files[0].type === "application/pdf") {
                    onFileSelect(files[0]);
                } else {
                    alert("PDF file only / PDFのみアップロード可能です。");
                }
            }
        },
        [onFileSelect]
    );

    const handleDragOver = useCallback((e: React.DragEvent<HTMLDivElement>) => {
        e.preventDefault();
        e.stopPropagation();
    }, []);

    const handleInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
        if (e.target.files && e.target.files.length > 0) {
            onFileSelect(e.target.files[0]);
        }
    }

    return (
        <div
            onDrop={handleDrop}
            onDragOver={handleDragOver}
            className={clsx(
                "border-2 border-dashed border-gray-300 rounded-lg p-12 text-center",
                "hover:border-blue-500 transition-colors cursor-pointer bg-gray-50"
            )}
        >
            <input
                type="file"
                accept=".pdf"
                className="hidden"
                id="file-upload"
                onChange={handleInputChange}
            />
            <label htmlFor="file-upload" className="cursor-pointer">
                <div className="text-gray-600">
                    <svg className="mx-auto h-12 w-12 text-gray-400" stroke="currentColor" fill="none" viewBox="0 0 48 48" aria-hidden="true">
                        <path d="M28 8H12a4 4 0 00-4 4v20m32-12v8m0 0v8a4 4 0 01-4 4H12a4 4 0 01-4-4v-4m32-4l-3.172-3.172a4 4 0 00-5.656 0L28 28M8 32l9.172-9.172a4 4 0 015.656 0L28 28m0 0l4 4m4-24h8m-4-4v8m-12 4h.02" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round" />
                    </svg>
                    <p className="mt-1 font-bold">{text}</p>
                    {subText && <p className="text-sm text-gray-400 mt-1">{subText}</p>}
                </div>
            </label>
        </div>
    );
};
