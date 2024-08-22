; (() => {
  Object.defineProperty(window, 'CUSTOM_PROTOCOL', {
    value: (protocol, url) => $get_protocol_url(),
  });

  function injectImportMap() {
    if (document.getElementById('lenz-import-map')) return;

    const imports = $get_import_map()
    const script = document.createElement('script')

    script.id = 'lenz-import-map'
    script.type = 'importmap'
    script.textContent = JSON.stringify({ imports })

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