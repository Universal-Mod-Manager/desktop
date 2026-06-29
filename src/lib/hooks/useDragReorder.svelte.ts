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
    scrollContainer: HTMLElement | null;
    initialScrollTop: number;
    latestClientY: number;
    initialRows: RowSnapshot[];
    rows: RowSnapshot[];
};

type DragRowAction = Action<HTMLElement, string>;

type AutoScrollIntent = {
    direction: -1 | 1;
    intensity: number;
};

function clamp(value: number, min: number, max: number) {
    return Math.min(Math.max(value, min), max);
}

export function useDragReorder({ onReorder }: DragReorderOptions) {
    let itemIds: string[] = [];
    let listNode: HTMLElement | null = null;
    let active = $state<ActiveDrag | null>(null);
    let isDropCommitting = $state(false);
    let dropCommitFrame: number | null = null;
    let autoScrollFrame: number | null = null;
    let autoScrollStartedAt: number | null = null;
    let autoScrollLastFrameAt: number | null = null;
    const rowNodes = new Map<string, HTMLElement>();

    const list: Action<HTMLElement> = (node) => {
        listNode = node;

        return {
            destroy() {
                if (active) {
                    endDrag(false);
                }
                if (listNode === node) {
                    listNode = null;
                }
                clearDropCommitFrame();
                stopAutoScroll();
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

        const scrollContainer = getScrollContainer();
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
            scrollContainer,
            initialScrollTop: scrollContainer?.scrollTop ?? 0,
            latestClientY: event.clientY,
            initialRows: rows,
            rows,
        };

        window.addEventListener("pointermove", handlePointerMove, { passive: false });
        window.addEventListener("pointerup", handlePointerUp);
        window.addEventListener("pointercancel", handlePointerCancel);
        window.addEventListener("keydown", handleKeyDown);
        window.addEventListener("blur", handleWindowBlur);
        scrollContainer?.addEventListener("scroll", handleScroll, { passive: true });

        updateDragPosition(event.clientY);
        updateAutoScroll();
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
        const scrollContainer = getScrollContainer();
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

    function getScrollContainer() {
        return listNode?.closest<HTMLElement>(".umm-content-body") ?? null;
    }

    function updateDragPosition(clientY: number) {
        if (!active) return;

        const drag = refreshDragMeasurements({
            ...active,
            latestClientY: clientY,
        });
        const top = clamp(
            clientY - drag.pointerOffsetY,
            drag.minTop,
            drag.maxTop,
        );
        const center = top + drag.height / 2;

        active = {
            ...drag,
            targetIndex: getTargetIndex(top, center, drag),
            y: top - drag.originalTop,
        };
    }

    function refreshDragMeasurements(drag: ActiveDrag): ActiveDrag {
        const scrollTop = drag.scrollContainer?.scrollTop ?? 0;
        const scrollDelta = scrollTop - drag.initialScrollTop;
        const rows = drag.initialRows.map((row) => ({
            ...row,
            top: row.top - scrollDelta,
            bottom: row.bottom - scrollDelta,
            center: row.center - scrollDelta,
        }));
        const draggedRow = rows.find((row) => row.id === drag.id);
        const bounds = getDragBounds(drag.height);

        return {
            ...drag,
            rows,
            originalTop: draggedRow?.top ?? drag.originalTop,
            minTop: bounds.minTop,
            maxTop: bounds.maxTop,
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
        updateAutoScroll();
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

    function handleScroll() {
        if (!active) return;

        updateDragPosition(active.latestClientY);
        updateAutoScroll();
    }

    function endDrag(shouldCommit: boolean) {
        const drag = active;
        if (!drag) return;
        const shouldReorder = shouldCommit && drag.targetIndex !== drag.fromIndex;

        if (shouldReorder) {
            disableTransitionsForDropCommit();
        }
        cleanupDragListeners(drag);
        active = null;

        if (drag.handle.hasPointerCapture(drag.pointerId)) {
            drag.handle.releasePointerCapture(drag.pointerId);
        }

        if (shouldReorder) {
            onReorder(drag.fromIndex, drag.targetIndex);
        }
    }

    function updateAutoScroll() {
        if (!active || !getAutoScrollIntent(active)) {
            stopAutoScroll();
            return;
        }

        startAutoScroll();
    }

    function startAutoScroll() {
        if (autoScrollFrame !== null) return;

        autoScrollStartedAt = null;
        autoScrollLastFrameAt = null;
        autoScrollFrame = window.requestAnimationFrame(runAutoScroll);
    }

    function runAutoScroll(timestamp: number) {
        autoScrollFrame = null;

        const drag = active;
        const intent = drag ? getAutoScrollIntent(drag) : null;
        if (!drag || !intent || !drag.scrollContainer) {
            stopAutoScroll();
            return;
        }

        if (autoScrollStartedAt === null) {
            autoScrollStartedAt = timestamp;
        }

        const frameDelta = autoScrollLastFrameAt === null
            ? 16
            : clamp(timestamp - autoScrollLastFrameAt, 0, 50);
        autoScrollLastFrameAt = timestamp;

        const scrollContainer = drag.scrollContainer;
        const maxScrollTop = getMaxScrollTop(scrollContainer);
        const beforeScrollTop = scrollContainer.scrollTop;
        const pixelsPerSecond = getCssNumber("--umm-drag-autoscroll-max-speed", 900);
        const ramp = getAutoScrollRamp(timestamp);
        const scrollDelta = intent.direction
            * intent.intensity
            * ramp
            * pixelsPerSecond
            * (frameDelta / 1000);

        scrollContainer.scrollTop = clamp(
            beforeScrollTop + scrollDelta,
            0,
            maxScrollTop,
        );

        if (scrollContainer.scrollTop === beforeScrollTop) {
            stopAutoScroll();
            return;
        }

        if (active) {
            updateDragPosition(active.latestClientY);
        }

        autoScrollFrame = window.requestAnimationFrame(runAutoScroll);
    }

    function stopAutoScroll() {
        if (autoScrollFrame !== null) {
            window.cancelAnimationFrame(autoScrollFrame);
            autoScrollFrame = null;
        }

        autoScrollStartedAt = null;
        autoScrollLastFrameAt = null;
    }

    function getAutoScrollIntent(drag: ActiveDrag): AutoScrollIntent | null {
        const scrollContainer = drag.scrollContainer;
        if (!scrollContainer) return null;

        const containerRect = scrollContainer.getBoundingClientRect();
        if (drag.height >= containerRect.height) return null;
        const maxScrollTop = getMaxScrollTop(scrollContainer);

        const edgeSize = getAutoScrollEdgeSize(containerRect.height);
        const draggedCenter = drag.originalTop + drag.y + drag.height / 2;
        const distanceFromTop = draggedCenter - containerRect.top;
        const distanceFromBottom = containerRect.bottom - draggedCenter;

        if (distanceFromTop < edgeSize && scrollContainer.scrollTop > 0) {
            return {
                direction: -1,
                intensity: getAutoScrollIntensity(distanceFromTop, edgeSize),
            };
        }

        if (
            distanceFromBottom < edgeSize
            && scrollContainer.scrollTop < maxScrollTop
        ) {
            return {
                direction: 1,
                intensity: getAutoScrollIntensity(distanceFromBottom, edgeSize),
            };
        }

        return null;
    }

    function getAutoScrollEdgeSize(containerHeight: number) {
        const ratio = getCssNumber("--umm-drag-autoscroll-edge-ratio", 0.18);
        const min = getCssNumber("--umm-drag-autoscroll-edge-min", 48);
        const max = getCssNumber("--umm-drag-autoscroll-edge-max", 120);

        return clamp(containerHeight * ratio, min, max);
    }

    function getAutoScrollIntensity(distanceFromEdge: number, edgeSize: number) {
        const maxSpeedZone = edgeSize * 0.75;
        const proximity = clamp((edgeSize - distanceFromEdge) / maxSpeedZone, 0, 1);

        return proximity * proximity;
    }

    function getAutoScrollRamp(timestamp: number) {
        const startedAt = autoScrollStartedAt ?? timestamp;
        const delay = getCssNumber("--umm-drag-autoscroll-ramp-delay", 250);
        const duration = getCssNumber("--umm-drag-autoscroll-ramp-duration", 1200);
        const elapsed = Math.max(0, timestamp - startedAt - delay);
        const progress = duration <= 0 ? 1 : clamp(elapsed / duration, 0, 1);

        return 0.35 + 0.65 * (progress * progress);
    }

    function getMaxScrollTop(element: HTMLElement) {
        return Math.max(0, element.scrollHeight - element.clientHeight);
    }

    function getCssNumber(name: string, fallback: number) {
        const source = listNode ?? document.documentElement;
        const raw = getComputedStyle(source).getPropertyValue(name).trim();
        const value = Number.parseFloat(raw);

        return Number.isFinite(value) ? value : fallback;
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

    function cleanupDragListeners(drag?: ActiveDrag) {
        window.removeEventListener("pointermove", handlePointerMove);
        window.removeEventListener("pointerup", handlePointerUp);
        window.removeEventListener("pointercancel", handlePointerCancel);
        window.removeEventListener("keydown", handleKeyDown);
        window.removeEventListener("blur", handleWindowBlur);
        drag?.scrollContainer?.removeEventListener("scroll", handleScroll);
        stopAutoScroll();
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
