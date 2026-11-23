export function captureDimensions(width: number, height: number) {
  if (
    ![width, height].every((n) => Number.isSafeInteger(n) && n > 0) ||
    width > 32767 ||
    height > 32767 ||
    width * height > 32_000_000
  )
    throw new Error(
      'Results image is too large to capture (maximum edge 32,767 pixels and area 32,000,000 pixels).',
    );
}

export async function captureResults(region: HTMLElement, stillCurrent: () => boolean) {
  const frame = () => new Promise<void>((resolve) => requestAnimationFrame(() => resolve()));
  await document.fonts.ready;
  await Promise.all(
    Array.from(region.querySelectorAll('img'), async (image) => {
      await image.decode();
      if (!image.naturalWidth) throw new Error('Results artwork could not be loaded.');
    }),
  );
  await frame();
  const size = () =>
    [
      Math.ceil(region.getBoundingClientRect().width),
      Math.ceil(region.getBoundingClientRect().height),
    ] as const;
  const [width, height] = size();
  await frame();
  if (!stillCurrent() || !region.isConnected || size()[0] !== width || size()[1] !== height)
    throw new Error('Results changed during export. Try again when the run and layout are stable.');
  captureDimensions(width, height);
  const { toSvg } = await import('html-to-image');
  let canvas: HTMLCanvasElement | undefined;
  const image = new Image();
  try {
    const svgUrl = await toSvg(region, {
      width,
      height,
      filter: (node) =>
        !(
          node instanceof Element && node.matches('[role="tooltip"], .chart-tooltip, .card-tooltip')
        ),
    });
    // The pinned library deep-clones SVG without styling its descendants.
    // Decorate only the detached SVG markup; never mutate live charts/cards.
    const xml = new DOMParser().parseFromString(
      decodeURIComponent(svgUrl.slice(svgUrl.indexOf(',') + 1)),
      'image/svg+xml',
    );
    const originals = Array.from(region.querySelectorAll('svg'));
    const copies = Array.from(xml.querySelectorAll('foreignObject svg'));
    if (originals.length !== copies.length) throw new Error('Results SVG capture is incomplete.');
    originals.forEach((original, index) => {
      const sourceNodes = [original, ...original.querySelectorAll('*')];
      const copyNodes = [copies[index], ...copies[index].querySelectorAll('*')];
      sourceNodes.forEach((source, childIndex) => {
        const copy = copyNodes[childIndex] as SVGElement;
        if (!copy?.style) return;
        const computed = getComputedStyle(source);
        for (const name of [
          'fill',
          'fill-opacity',
          'stroke',
          'stroke-width',
          'stroke-opacity',
          'stroke-dasharray',
          'stroke-linecap',
          'stroke-linejoin',
          'opacity',
          'font-family',
          'font-size',
          'font-weight',
          'letter-spacing',
          'text-anchor',
          'paint-order',
          'visibility',
        ])
          copy.style.setProperty(name, computed.getPropertyValue(name));
      });
    });
    image.src =
      'data:image/svg+xml;charset=utf-8,' +
      encodeURIComponent(new XMLSerializer().serializeToString(xml));
    await image.decode();
    canvas = document.createElement('canvas');
    canvas.width = width;
    canvas.height = height;
    const context = canvas.getContext('2d');
    if (!context || canvas.width !== width || canvas.height !== height)
      throw new Error('Browser could not capture the complete Results image.');
    context.fillStyle = getComputedStyle(document.body).backgroundColor;
    context.fillRect(0, 0, width, height);
    context.drawImage(image, 0, 0);
    const blob = await new Promise<Blob>((resolve, reject) =>
      canvas!.toBlob(
        (value) =>
          value ? resolve(value) : reject(new Error('Browser could not encode the Results PNG.')),
        'image/png',
      ),
    );
    if (!stillCurrent()) throw new Error('Results changed during export. Try again.');
    return blob;
  } finally {
    image.src = '';
    if (canvas) {
      canvas.width = 0;
      canvas.height = 0;
      canvas.remove();
    }
  }
}
