/**
 * Coordinate picker utilities for screen operations
 */

export interface Coordinate {
  x: number;
  y: number;
}

export interface RGBColor {
  r: number;
  g: number;
  b: number;
}

export interface Region {
  startX: number;
  startY: number;
  endX: number;
  endY: number;
}

/**
 * Extract RGB color from canvas at coordinates
 */
export const extractRGBAtCoordinate = (
  canvas: HTMLCanvasElement,
  x: number,
  y: number
): RGBColor | null => {
  try {
    const ctx = canvas.getContext('2d', { willReadFrequently: true });
    if (!ctx) return null;

    const imageData = ctx.getImageData(x, y, 1, 1);
    const data = imageData.data;

    return {
      r: data[0],
      g: data[1],
      b: data[2],
    };
  } catch (e) {
    console.error('Failed to extract RGB:', e);
    return null;
  }
};

/**
 * Normalize region coordinates (swap if needed)
 */
export const normalizeRegion = (region: Region): Region => {
  return {
    startX: Math.min(region.startX, region.endX),
    startY: Math.min(region.startY, region.endY),
    endX: Math.max(region.startX, region.endX),
    endY: Math.max(region.startY, region.endY),
  };
};

/**
 * Calculate region dimensions
 */
export const getRegionDimensions = (region: Region): { width: number; height: number } => {
  const normalized = normalizeRegion(region);
  return {
    width: normalized.endX - normalized.startX,
    height: normalized.endY - normalized.startY,
  };
};

/**
 * Convert canvas coordinates to image coordinates based on zoom and pan
 */
export const canvasToImageCoordinates = (
  canvasX: number,
  canvasY: number,
  zoom: number,
  panX: number,
  panY: number
): Coordinate => {
  return {
    x: Math.round((canvasX - panX) / zoom),
    y: Math.round((canvasY - panY) / zoom),
  };
};

/**
 * Convert image coordinates to canvas coordinates
 */
export const imageToCanvasCoordinates = (
  imageX: number,
  imageY: number,
  zoom: number,
  panX: number,
  panY: number
): Coordinate => {
  return {
    x: imageX * zoom + panX,
    y: imageY * zoom + panY,
  };
};

/**
 * Format coordinate as string
 */
export const formatCoordinate = (coord: Coordinate): string => {
  return `${coord.x},${coord.y}`;
};

/**
 * Format coordinate as JSON
 */
export const formatCoordinateJSON = (coord: Coordinate): string => {
  return JSON.stringify({ x: coord.x, y: coord.y });
};

/**
 * Format RGB color as hex string
 */
export const formatRGBAsHex = (color: RGBColor): string => {
  const toHex = (n: number) => {
    const hex = n.toString(16);
    return hex.length === 1 ? '0' + hex : hex;
  };
  return `#${toHex(color.r)}${toHex(color.g)}${toHex(color.b)}`;
};

/**
 * Format RGB color as CSS string
 */
export const formatRGBAsCSS = (color: RGBColor): string => {
  return `rgb(${color.r}, ${color.g}, ${color.b})`;
};

/**
 * Create image region canvas
 */
export const createRegionCanvas = (
  sourceCanvas: HTMLCanvasElement,
  region: Region
): HTMLCanvasElement | null => {
  try {
    const normalized = normalizeRegion(region);
    const { width, height } = getRegionDimensions(region);

    if (width <= 0 || height <= 0) return null;

    const regionCanvas = document.createElement('canvas');
    regionCanvas.width = width;
    regionCanvas.height = height;

    const ctx = regionCanvas.getContext('2d');
    if (!ctx) return null;

    const sourceCtx = sourceCanvas.getContext('2d');
    if (!sourceCtx) return null;

    const imageData = sourceCtx.getImageData(
      normalized.startX,
      normalized.startY,
      width,
      height
    );
    ctx.putImageData(imageData, 0, 0);

    return regionCanvas;
  } catch (e) {
    console.error('Failed to create region canvas:', e);
    return null;
  }
};

/**
 * Check if point is within region
 */
export const isPointInRegion = (point: Coordinate, region: Region): boolean => {
  const normalized = normalizeRegion(region);
  return (
    point.x >= normalized.startX &&
    point.x <= normalized.endX &&
    point.y >= normalized.startY &&
    point.y <= normalized.endY
  );
};
