import type { Action } from "svelte/action";

type DragReorderOptions = {
    onReorder: (fromIndex: number, toIndex: number) => void;
};

type RowSnapshot = {
    id: string;
    index: number;
    top: number;
    bottom: number;
    height: number;
    center: number;
};

type ActiveDrag = {
    id: string;
    pointerId: number;
    fromIndex: number;
    targetIndex: number;
    pointerOffsetY: number;
    originalTop: number;
    height: number;
    minTop: number;
    maxTop: number;
    shiftUp: number;
    shiftDown: number;
    y: number;
    handle: HTMLElement;
    rows: RowSnapshot[];
};

type DragRowAction = Action<HTMLElement, string>;

function clamp(value: number, min: number, max: number) {
    return Math.min(Math.max(value, min), max);
}

export function useDragReorder({ onReorder }: DragReorderOptions) {
    let itemIds: string[] = [];
    let listNode: HTMLElement | null = null;
    let active = $state<ActiveDrag | null>(null);
    let isDropCommitting = $state(false);
    let dropCommitFrame: number | null = null;
    const rowNodes = new Map<string, HTMLElement>();

    const list: Action<HTMLElement> = (node) => {
        listNode = node;

        return {
            destroy() {
                if (listNode === node) {
                    listNode = null;
                }
                clearDropCommitFrame();
            },
        };
    };

    const row: DragRowAction = (node, id) => {
        rowNodes.set(id, node);

        return {
            update(nextId) {
                if (nextId === id) return;

                if (rowNodes.get(id) === node) {
                    rowNodes.delete(id);
                }

                id = nextId;
                rowNodes.set(id, node);
            },
            destroy() {
                if (rowNodes.get(id) === node) {
                    rowNodes.delete(id);
                }
            },
        };
    };

    function setItems(ids: string[]) {
        itemIds = ids;

        if (active && !ids.includes(active.id)) {
            endDrag(false);
        }
    }

    function startDrag(id: string, event: PointerEvent) {
        if (event.button !== 0 || active) return;

        const handle = event.currentTarget;
        if (!(handle instanceof HTMLElement)) return;

        const fromIndex = itemIds.indexOf(id);
        const draggedRow = rowNodes.get(id);
        if (fromIndex === -1 || !draggedRow) return;

        const rows = measureRows();
        const draggedSnapshot = rows.find((snapshot) => snapshot.id === id);
        if (!draggedSnapshot) return;

        const bounds = getDragBounds(draggedSnapshot.height);
        const previousRow = rows[fromIndex - 1];
        const nextRow = rows[fromIndex + 1];
        const gapBefore = previousRow
            ? Math.max(0, draggedSnapshot.top - previousRow.bottom)
            : 0;
        const gapAfter = nextRow
            ? Math.max(0, nextRow.top - draggedSnapshot.bottom)
            : gapBefore;

        event.preventDefault();
        event.stopPropagation();

        handle.setPointerCapture(event.pointerId);

        active = {
            id,
            pointerId: event.pointerId,
            fromIndex,
            targetIndex: fromIndex,
            pointerOffsetY: event.clientY - draggedSnapshot.top,
            originalTop: draggedSnapshot.top,
            height: draggedSnapshot.height,
            minTop: bounds.minTop,
            maxTop: bounds.maxTop,
            shiftUp: draggedSnapshot.height + gapBefore,
            shiftDown: draggedSnapshot.height + gapAfter,
            y: 0,
            handle,
            rows,
        };

        window.addEventListener("pointermove", handlePointerMove, { passive: false });
        window.addEventListener("pointerup", handlePointerUp);
        window.addEventListener("pointercancel", handlePointerCancel);
        window.addEventListener("keydown", handleKeyDown);
        window.addEventListener("blur", handleWindowBlur);

        updateDragPosition(event.clientY);
    }

    function measureRows() {
        return itemIds
            .map((id, index) => {
                const node = rowNodes.get(id);
                if (!node) return null;

                const rect = node.getBoundingClientRect();
                return {
                    id,
                    index,
                    top: rect.top,
                    bottom: rect.bottom,
                    height: rect.height,
                    center: rect.top + rect.height / 2,
                };
            })
            .filter((row): row is RowSnapshot => row !== null);
    }

    function getDragBounds(rowHeight: number) {
        const listRect = listNode?.getBoundingClientRect();
        const scrollContainer = listNode?.closest<HTMLElement>(".umm-content-body");
        const containerRect = scrollContainer?.getBoundingClientRect();
        const top = Math.max(
            listRect?.top ?? Number.NEGATIVE_INFINITY,
            containerRect?.top ?? Number.NEGATIVE_INFINITY,
        );
        const bottom = Math.min(
            listRect?.bottom ?? Number.POSITIVE_INFINITY,
            containerRect?.bottom ?? Number.POSITIVE_INFINITY,
        );
        const minTop = Number.isFinite(top) ? top : 0;
        const maxTop = Number.isFinite(bottom) ? Math.max(minTop, bottom - rowHeight) : minTop;

        return { minTop, maxTop };
    }

    function updateDragPosition(clientY: number) {
        if (!active) return;

        const top = clamp(
            clientY - active.pointerOffsetY,
            active.minTop,
            active.maxTop,
        );
        const center = top + active.height / 2;

        active = {
            ...active,
            targetIndex: getTargetIndex(top, center, active),
            y: top - active.originalTop,
        };
    }

    function getTargetIndex(top: number, centerY: number, drag: ActiveDrag) {
        const rowsWithoutDragged = drag.rows.filter((row) => row.id !== drag.id);
        const firstRow = rowsWithoutDragged[0];
        const lastRow = rowsWithoutDragged[rowsWithoutDragged.length - 1];

        if (!firstRow || !lastRow) return 0;

        if (top <= firstRow.top) return 0;
        if (top + drag.height >= lastRow.bottom) return rowsWithoutDragged.length;

        for (let index = 0; index < rowsWithoutDragged.length; index += 1) {
            if (centerY < rowsWithoutDragged[index].center) {
                return index;
            }
        }

        return rowsWithoutDragged.length;
    }

    function handlePointerMove(event: PointerEvent) {
        if (!active || event.pointerId !== active.pointerId) return;

        event.preventDefault();
        updateDragPosition(event.clientY);
    }

    function handlePointerUp(event: PointerEvent) {
        if (!active || event.pointerId !== active.pointerId) return;

        event.preventDefault();
        endDrag(true);
    }

    function handlePointerCancel(event: PointerEvent) {
        if (!active || event.pointerId !== active.pointerId) return;

        endDrag(false);
    }

    function handleKeyDown(event: KeyboardEvent) {
        if (event.key === "Escape") {
            endDrag(false);
        }
    }

    function handleWindowBlur() {
        endDrag(false);
    }

    function endDrag(shouldCommit: boolean) {
        const drag = active;
        if (!drag) return;
        const shouldReorder = shouldCommit && drag.targetIndex !== drag.fromIndex;

        if (shouldReorder) {
            disableTransitionsForDropCommit();
        }
        active = null;
        cleanupDragListeners();

        if (drag.handle.hasPointerCapture(drag.pointerId)) {
            drag.handle.releasePointerCapture(drag.pointerId);
        }

        if (shouldReorder) {
            onReorder(drag.fromIndex, drag.targetIndex);
        }
    }

    function disableTransitionsForDropCommit() {
        clearDropCommitFrame();
        isDropCommitting = true;

        dropCommitFrame = window.requestAnimationFrame(() => {
            dropCommitFrame = window.requestAnimationFrame(() => {
                isDropCommitting = false;
                dropCommitFrame = null;
            });
        });
    }

    function clearDropCommitFrame() {
        if (dropCommitFrame === null) return;

        window.cancelAnimationFrame(dropCommitFrame);
        dropCommitFrame = null;
        isDropCommitting = false;
    }

    function cleanupDragListeners() {
        window.removeEventListener("pointermove", handlePointerMove);
        window.removeEventListener("pointerup", handlePointerUp);
        window.removeEventListener("pointercancel", handlePointerCancel);
        window.removeEventListener("keydown", handleKeyDown);
        window.removeEventListener("blur", handleWindowBlur);
    }

    function getShiftOffset(id: string) {
        if (!active || id === active.id) return 0;

        const row = active.rows.find((snapshot) => snapshot.id === id);
        if (!row) return 0;

        if (active.targetIndex > active.fromIndex) {
            return row.index > active.fromIndex && row.index <= active.targetIndex
                ? -active.shiftDown
                : 0;
        }

        if (active.targetIndex < active.fromIndex) {
            return row.index >= active.targetIndex && row.index < active.fromIndex
                ? active.shiftUp
                : 0;
        }

        return 0;
    }

    function getRowStyle(id: string) {
        if (!active) return undefined;

        const y = id === active.id ? active.y : getShiftOffset(id);
        return y === 0 ? undefined : `--umm-drag-y: ${y}px;`;
    }

    return {
        list,
        row,
        setItems,
        startDrag,
        get isDragging() {
            return active !== null;
        },
        get isDropCommitting() {
            return isDropCommitting;
        },
        isDraggingRow(id: string) {
            return active?.id === id;
        },
        getRowStyle,
    };
}
