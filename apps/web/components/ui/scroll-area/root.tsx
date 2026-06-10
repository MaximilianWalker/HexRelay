"use client";

import { useCallback, useEffect, useRef, useState } from "react";
import type { CSSProperties, HTMLAttributes, ReactNode } from "react";

import { cx } from "@/lib/ui/cx";

import styles from "./styles.module.css";

type ScrollMetrics = {
  canScroll: boolean;
  thumbOffset: number;
  thumbSize: number;
};

export function Root({
  children,
  className,
  hideDelayMs = 1200,
  hideWhenIdle = false,
  overlay = true,
  viewportClassName,
  width = 4,
  ...props
}: HTMLAttributes<HTMLDivElement> & {
  children: ReactNode;
  hideDelayMs?: number;
  hideWhenIdle?: boolean;
  overlay?: boolean;
  viewportClassName?: string;
  width?: number;
}) {
  const viewportRef = useRef<HTMLDivElement>(null);
  const hideTimerRef = useRef<number | null>(null);
  const [idle, setIdle] = useState(false);
  const [metrics, setMetrics] = useState<ScrollMetrics>({
    canScroll: false,
    thumbOffset: 0,
    thumbSize: 0,
  });

  const resolvedWidth = Number.isFinite(width) ? Math.max(1, width) : 4;
  const resolvedHideDelayMs = Number.isFinite(hideDelayMs) ? Math.max(0, hideDelayMs) : 1200;

  const clearHideTimer = useCallback(() => {
    if (hideTimerRef.current === null) {
      return;
    }

    window.clearTimeout(hideTimerRef.current);
    hideTimerRef.current = null;
  }, []);

  const markActive = useCallback(() => {
    if (!hideWhenIdle) {
      setIdle(false);
      clearHideTimer();
      return;
    }

    setIdle(false);
    clearHideTimer();
    hideTimerRef.current = window.setTimeout(() => {
      setIdle(true);
      hideTimerRef.current = null;
    }, resolvedHideDelayMs);
  }, [clearHideTimer, hideWhenIdle, resolvedHideDelayMs]);

  const updateMetrics = useCallback(() => {
    const viewport = viewportRef.current;

    if (!viewport) {
      return;
    }

    const { clientHeight, scrollHeight, scrollTop } = viewport;
    const canScroll = scrollHeight > clientHeight + 1;

    if (!canScroll) {
      clearHideTimer();
      setIdle(false);
      setMetrics((current) =>
        current.canScroll || current.thumbOffset !== 0 || current.thumbSize !== 0
          ? { canScroll: false, thumbOffset: 0, thumbSize: 0 }
          : current,
      );
      return;
    }

    markActive();

    const minThumbSize = Math.min(clientHeight, 40);
    const thumbSize = Math.max(minThumbSize, Math.round((clientHeight / scrollHeight) * clientHeight));
    const maxThumbOffset = clientHeight - thumbSize;
    const maxScrollTop = scrollHeight - clientHeight;
    const thumbOffset = maxScrollTop > 0 ? Math.round((scrollTop / maxScrollTop) * maxThumbOffset) : 0;

    setMetrics((current) =>
      current.canScroll === canScroll && current.thumbOffset === thumbOffset && current.thumbSize === thumbSize
        ? current
        : { canScroll, thumbOffset, thumbSize },
    );
  }, [clearHideTimer, markActive]);

  useEffect(() => {
    const viewport = viewportRef.current;

    if (!viewport) {
      return;
    }

    let animationFrame = window.requestAnimationFrame(updateMetrics);
    const resizeObserver =
      typeof ResizeObserver === "undefined"
        ? null
        : new ResizeObserver(() => {
            window.cancelAnimationFrame(animationFrame);
            animationFrame = window.requestAnimationFrame(updateMetrics);
          });

    function handleScroll(): void {
      window.cancelAnimationFrame(animationFrame);
      updateMetrics();
    }

    viewport.addEventListener("scroll", handleScroll, { passive: true });
    viewport.addEventListener("pointerenter", markActive);
    viewport.addEventListener("pointermove", markActive);
    window.addEventListener("resize", handleScroll);
    resizeObserver?.observe(viewport);
    if (viewport.firstElementChild) {
      resizeObserver?.observe(viewport.firstElementChild);
    }

    return () => {
      window.cancelAnimationFrame(animationFrame);
      viewport.removeEventListener("scroll", handleScroll);
      viewport.removeEventListener("pointerenter", markActive);
      viewport.removeEventListener("pointermove", markActive);
      window.removeEventListener("resize", handleScroll);
      resizeObserver?.disconnect();
      clearHideTimer();
    };
  }, [clearHideTimer, markActive, updateMetrics]);

  const rootStyle = {
    "--scroll-area-thumb-offset": `${metrics.thumbOffset}px`,
    "--scroll-area-thumb-size": `${metrics.thumbSize}px`,
    "--scroll-area-thumb-width": `${resolvedWidth}px`,
    "--scroll-area-track-width": `${resolvedWidth + 4}px`,
    ...props.style,
  } as CSSProperties;
  const scrollbarVisible = metrics.canScroll && (!hideWhenIdle || !idle);

  return (
    <div
      {...props}
      className={cx(styles.scrollArea, className)}
      data-hide-when-idle={hideWhenIdle ? "true" : undefined}
      data-overlay={overlay ? "true" : "false"}
      data-scrollbar-visible={scrollbarVisible ? "true" : undefined}
      data-testid="scroll-area"
      style={rootStyle}
    >
      <div className={cx(styles.viewport, viewportClassName)} data-testid="scroll-area-viewport" ref={viewportRef}>
        {children}
      </div>
      <div aria-hidden="true" className={styles.track}>
        <span className={styles.thumb} />
      </div>
    </div>
  );
}
