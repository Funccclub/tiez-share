// Original, dependency-free edge lens. Only two fixed controls use SVG refraction.
// Safari/Firefox, touch devices and accessibility modes keep the CSS material.
(() => {
    if (!document.body.classList.contains('theme-liquid-glass')) return;
    const icon = path => '<svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">' + path + '</svg>';
    const folder = icon('<path d="M3 7a2 2 0 0 1 2-2h5l2 2h7a2 2 0 0 1 2 2v9a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2Z"/>');
    const filesButton = document.getElementById('files-toggle');
    filesButton.innerHTML = folder;
    filesButton.setAttribute('aria-label', '显示或隐藏共享文件');
    const sendButton = document.getElementById('send-btn');
    sendButton.innerHTML = icon('<path d="m6 11 6-6 6 6M12 5v14"/>');
    sendButton.title = '发送';
    sendButton.setAttribute('aria-label', '发送消息');
    document.querySelector('#file-panel-head > span').innerHTML = folder + '<span>共享文件</span>';
    const closeFiles = document.createElement('button');
    closeFiles.type = 'button';
    closeFiles.className = 'glass-panel-close';
    closeFiles.innerHTML = icon('<path d="m6 6 12 12M18 6 6 18"/>');
    closeFiles.setAttribute('aria-label', '关闭共享文件');
    closeFiles.onclick = () => { document.body.classList.remove('panel-open'); filesButton.focus(); };
    document.getElementById('file-panel-head').appendChild(closeFiles);
    document.addEventListener('keydown', event => {
        if (event.key === 'Escape' && document.body.classList.contains('panel-open')) closeFiles.click();
    });
    document.addEventListener('click', event => {
        if (!document.getElementById('file-panel').contains(event.target) && !filesButton.contains(event.target)) document.body.classList.remove('panel-open');
    });
    document.querySelector('#fs-editor > div > span').textContent = '全屏编辑';
    document.querySelector('.fs-toolbar button').textContent = '返回';
    document.querySelector('.fs-toolbar .send-btn').textContent = '发送';
    document.querySelector('.add-btn').title = '选择文件上传';
    const transparency = matchMedia('(prefers-reduced-transparency: reduce)');
    const contrast = matchMedia('(forced-colors: active)');
    const touch = matchMedia('(pointer: coarse)');
    const chromium = /(?:Chrome|Chromium|Edg)\//.test(navigator.userAgent);
    if (!chromium || !CSS.supports('backdrop-filter', 'url(#glass) blur(1px)') || !window.ResizeObserver) return;
    const ns = 'http://www.w3.org/2000/svg';
    const svg = document.createElementNS(ns, 'svg');
    svg.setAttribute('aria-hidden', 'true');
    svg.style.cssText = 'position:absolute;width:0;height:0;pointer-events:none;';
    const defs = document.createElementNS(ns, 'defs');
    svg.appendChild(defs);
    document.body.appendChild(svg);
    const surfaces = [...document.querySelectorAll('header, footer')].map((el, index) => {
        const filter = document.createElementNS(ns, 'filter');
        filter.id = 'tiez-glass-' + index;
        filter.setAttribute('filterUnits', 'userSpaceOnUse');
        filter.setAttribute('color-interpolation-filters', 'sRGB');
        const image = document.createElementNS(ns, 'feImage');
        image.setAttribute('result', 'lens');
        image.setAttribute('preserveAspectRatio', 'none');
        const displacement = document.createElementNS(ns, 'feDisplacementMap');
        for (const [key, value] of Object.entries({ in: 'SourceGraphic', in2: 'lens', scale: '12', xChannelSelector: 'R', yChannelSelector: 'G' })) displacement.setAttribute(key, value);
        filter.append(image, displacement);
        defs.appendChild(filter);
        el.style.setProperty('--glass-filter', 'url(#' + filter.id + ')');
        return { el, filter, image, size: '' };
    });
    function refresh() {
        const enabled = !transparency.matches && !contrast.matches && !touch.matches;
        for (const surface of surfaces) {
            const { el, filter, image } = surface;
            el.classList.toggle('glass-refractive', enabled);
            if (!enabled) continue;
            const { width, height } = el.getBoundingClientRect();
            const radius = parseFloat(getComputedStyle(el).borderTopLeftRadius) || 24;
            const size = [width, height, radius].join(',');
            if (!width || !height || size === surface.size) continue;
            surface.size = size;
            // Cap the map resolution. Rebuild only on resize, never on pointer/scroll.
            const canvas = document.createElement('canvas');
            canvas.width = Math.min(1024, Math.ceil(width));
            canvas.height = Math.min(160, Math.ceil(height));
            const ctx = canvas.getContext('2d');
            if (!ctx) { el.classList.remove('glass-refractive'); continue; }
            const pixels = ctx.createImageData(canvas.width, canvas.height);
            const r = Math.min(radius, width / 2, height / 2);
            for (let y = 0; y < canvas.height; y++) {
                for (let x = 0; x < canvas.width; x++) {
                    const px = (x + .5) * width / canvas.width - width / 2;
                    const py = (y + .5) * height / canvas.height - height / 2;
                    const qx = Math.abs(px) - width / 2 + r;
                    const qy = Math.abs(py) - height / 2 + r;
                    const ax = Math.max(qx, 0), ay = Math.max(qy, 0);
                    const length = Math.hypot(ax, ay);
                    const depth = -(length + Math.min(Math.max(qx, qy), 0) - r);
                    const bend = depth >= 0 ? Math.pow(Math.max(0, 1 - depth / 12), 2) : 0;
                    const nx = length ? ax / length : (qx > qy ? 1 : 0);
                    const ny = length ? ay / length : (qy >= qx ? 1 : 0);
                    const i = (y * canvas.width + x) * 4;
                    pixels.data[i] = 128 + Math.sign(px) * nx * bend * 110;
                    pixels.data[i + 1] = 128 + Math.sign(py) * ny * bend * 110;
                    pixels.data[i + 2] = 128;
                    pixels.data[i + 3] = 255;
                }
            }
            ctx.putImageData(pixels, 0, 0);
            for (const target of [filter, image]) {
                target.setAttribute('x', '0'); target.setAttribute('y', '0');
                target.setAttribute('width', String(width)); target.setAttribute('height', String(height));
            }
            image.setAttribute('href', canvas.toDataURL());
        }
    }
    let frame = 0;
    const schedule = () => {
        if (!frame) frame = requestAnimationFrame(() => { frame = 0; refresh(); });
    };
    const observer = new ResizeObserver(schedule);
    surfaces.forEach(({ el }) => observer.observe(el));
    [transparency, contrast, touch].forEach(query => query.addEventListener('change', schedule));
    schedule();
})();
