; (() => {
  Object.defineProperty(window, 'CUSTOM_PROTOCOL', {
    value: (protocol, url) => $get_protocol_url(),
  });

  function injectImportMap() {
    if (document.getElementById('lenz_importmap')) return;

    const script = document.createElement('script')

    script.id = 'lenz_importmap'
    script.type = 'importmap'
    script.textContent = JSON.stringify({ imports: $get_import_map() })

    document.head.appendChild(script)
  }

  function injectIpc() {
    if (document.getElementById('lenz-ipc')) return;

    const script = document.createElement('script')

    script.id = 'lenz-ipc'
    script.type = 'module'
    script.textContent = `import 'lenz/ipc';`

    document.head.appendChild(script)
  }

  const obs = new MutationObserver((mutations) => {
    for (const mutation of mutations) {
      for (const node of mutation.addedNodes) {
        if (node instanceof HTMLHeadElement) {
          injectImportMap()
          injectIpc()
          obs.disconnect()
        }
      }
    }
  })

  obs.observe(document.documentElement, { childList: true, subtree: true })
})()