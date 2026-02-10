import React, { useEffect, useRef, useState } from 'react';
import * as pdfjsLib from 'pdfjs-dist';
import { renderPageToCanvas } from '../utils/pdfRenderer';

interface MaskRect {
    page: number;
    x: number;
    y: number;
    width: number;
    height: number;
    color: 'black' | 'white';
}

interface PdfCanvasProps {
    pdfDoc: pdfjsLib.PDFDocumentProxy;
    pageNumber: number;
    masks: MaskRect[];
    onMasksChange: (masks: MaskRect[]) => void;
    activeTool: 'black' | 'white' | 'move';
    selectedMaskIndex: number | null;
    onSelectionChange: (index: number | null) => void;
}

export const PdfCanvas: React.FC<PdfCanvasProps> = ({
    pdfDoc, pageNumber, masks, onMasksChange,
    activeTool, selectedMaskIndex, onSelectionChange
}) => {
    const canvasRef = useRef<HTMLCanvasElement>(null);
    const overlayRef = useRef<HTMLDivElement>(null);
    const [viewport, setViewport] = useState<pdfjsLib.PageViewport | null>(null);

    // Interaction states
    const [interactionMode, setInteractionMode] = useState<'none' | 'drawing' | 'moving'>('none');
    const [startPos, setStartPos] = useState({ x: 0, y: 0 });
    const [currentRect, setCurrentRect] = useState<{ x: number, y: number, w: number, h: number } | null>(null);
    const [dragOffset, setDragOffset] = useState({ x: 0, y: 0 });

    useEffect(() => {
        if (canvasRef.current && pdfDoc) {
            renderPageToCanvas(pdfDoc, pageNumber, canvasRef.current).then(vp => {
                setViewport(vp);
            });
        }
    }, [pdfDoc, pageNumber]);

    const getRelativeCoords = (e: React.MouseEvent | MouseEvent) => {
        if (!overlayRef.current) return { x: 0, y: 0 };
        const rect = overlayRef.current.getBoundingClientRect();
        return {
            x: e.clientX - rect.left,
            y: e.clientY - rect.top
        };
    };

    const handleMouseDown = (e: React.MouseEvent) => {
        const pos = getRelativeCoords(e);

        // Find if clicked on any mask on this page
        let clickedMaskGlobalIndex = -1;
        if (viewport) {
            clickedMaskGlobalIndex = masks.findIndex((mask) => {
                if (mask.page !== pageNumber) return false;
                const rect = viewport.convertToViewportRectangle([mask.x, mask.y, mask.x + mask.width, mask.y + mask.height]);
                const left = Math.min(rect[0], rect[2]);
                const top = Math.min(rect[1], rect[3]);
                const right = Math.max(rect[0], rect[2]);
                const bottom = Math.max(rect[1], rect[3]);
                return pos.x >= left && pos.x <= right && pos.y >= top && pos.y <= bottom;
            });
        }

        if (activeTool === 'move' || clickedMaskGlobalIndex !== -1) {
            if (clickedMaskGlobalIndex !== -1) {
                const mask = masks[clickedMaskGlobalIndex];
                onSelectionChange(clickedMaskGlobalIndex);
                setInteractionMode('moving');
                setStartPos(pos);

                const rect = viewport!.convertToViewportRectangle([mask.x, mask.y, mask.x + mask.width, mask.y + mask.height]);
                setDragOffset({ x: pos.x - Math.min(rect[0], rect[2]), y: pos.y - Math.min(rect[1], rect[3]) });
                return;
            }
        }

        // Default to drawing if not clicking a mask or in drawing mode
        if (activeTool === 'black' || activeTool === 'white') {
            onSelectionChange(null);
            setInteractionMode('drawing');
            setStartPos(pos);
            setCurrentRect({ x: pos.x, y: pos.y, w: 0, h: 0 });
        } else {
            onSelectionChange(null);
        }
    };

    const handleMouseMove = (e: React.MouseEvent) => {
        if (interactionMode === 'none') return;
        const pos = getRelativeCoords(e);

        if (interactionMode === 'drawing') {
            setCurrentRect({
                x: Math.min(pos.x, startPos.x),
                y: Math.min(pos.y, startPos.y),
                w: Math.abs(pos.x - startPos.x),
                h: Math.abs(pos.y - startPos.y)
            });
        } else if (interactionMode === 'moving' && selectedMaskIndex !== null && viewport) {
            const mask = masks[selectedMaskIndex];
            const newX = pos.x - dragOffset.x;
            const newY = pos.y - dragOffset.y;

            const rect = viewport.convertToViewportRectangle([mask.x, mask.y, mask.x + mask.width, mask.y + mask.height]);
            const dw = Math.abs(rect[2] - rect[0]);
            const dh = Math.abs(rect[3] - rect[1]);

            const p1 = viewport.convertToPdfPoint(newX, newY);
            const p2 = viewport.convertToPdfPoint(newX + dw, newY + dh);

            const newMasks = [...masks];
            newMasks[selectedMaskIndex] = {
                ...mask,
                x: Math.min(p1[0], p2[0]),
                y: Math.min(p1[1], p2[1]),
                width: Math.abs(p1[0] - p2[0]),
                height: Math.abs(p1[1] - p2[1])
            };
            onMasksChange(newMasks);
        }
    };

    const handleMouseUp = () => {
        if (interactionMode === 'drawing' && currentRect && currentRect.w > 5 && currentRect.h > 5 && viewport) {
            const p1 = viewport.convertToPdfPoint(currentRect.x, currentRect.y);
            const p2 = viewport.convertToPdfPoint(currentRect.x + currentRect.w, currentRect.y + currentRect.h);

            const newMask: MaskRect = {
                page: pageNumber,
                x: Math.min(p1[0], p2[0]),
                y: Math.min(p1[1], p2[1]),
                width: Math.abs(p1[0] - p2[0]),
                height: Math.abs(p1[1] - p2[1]),
                color: activeTool === 'white' ? 'white' : 'black'
            };

            onMasksChange([...masks, newMask]);
        }

        setInteractionMode('none');
        setCurrentRect(null);
    };

    const renderMasks = () => {
        if (!viewport) return null;

        return masks.map((mask, i) => {
            if (mask.page !== pageNumber) return null;

            const rect = viewport.convertToViewportRectangle([mask.x, mask.y, mask.x + mask.width, mask.y + mask.height]);
            const isSelected = selectedMaskIndex === i;

            const style: React.CSSProperties = {
                left: Math.min(rect[0], rect[2]),
                top: Math.min(rect[1], rect[3]),
                width: Math.abs(rect[0] - rect[2]),
                height: Math.abs(rect[1] - rect[3]),
                position: 'absolute',
                backgroundColor: mask.color === 'white' ? 'white' : 'black',
                border: isSelected ? '2px solid #3b82f6' : (mask.color === 'white' ? '1px solid #ddd' : 'none'),
                cursor: activeTool === 'move' ? 'move' : 'pointer',
                boxSizing: 'border-box',
                opacity: isSelected ? 0.8 : 1.0
            };

            return (
                <div
                    key={i}
                    style={style}
                    onClick={(e) => { e.stopPropagation(); onSelectionChange(i); }}
                    title={mask.color === 'white' ? '白塗り' : '黒塗り'}
                />
            );
        });
    };

    return (
        <div
            className="relative inline-block border shadow-2xl bg-white select-none"
            ref={overlayRef}
            onMouseDown={handleMouseDown}
            onMouseMove={handleMouseMove}
            onMouseUp={handleMouseUp}
        >
            <canvas ref={canvasRef} className="block shadow-inner" />

            {/* Interaction Layer */}
            <div className="absolute top-0 left-0 w-full h-full pointer-events-none">
                {renderMasks()}
                {currentRect && (
                    <div
                        style={{
                            left: currentRect.x,
                            top: currentRect.y,
                            width: currentRect.w,
                            height: currentRect.h,
                            position: 'absolute',
                            border: '1px dashed #3b82f6',
                            backgroundColor: activeTool === 'white' ? 'rgba(255, 255, 255, 0.5)' : 'rgba(0, 0, 0, 0.5)'
                        }}
                    />
                )}
            </div>
        </div>
    );
};
