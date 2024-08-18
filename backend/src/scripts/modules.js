
(async () => {
  /** get_extensions() é uma macro. */
  window.EXTENSIONS = get_extensions();

  function createImportMapScript() {
    const script = document.createElement('script');

    script.type = 'importmap';


    /** get_import_map() é uma macro. */
    script.textContent = JSON.stringify({
      imports: get_import_map()
    });

    return script;
  }

  function createIpcScript() {
    const script = document.createElement('script');

    script.type = 'module';
    script.textContent = `import 'lenz/ipc';`;
    return script;
  }

  const observer = new MutationObserver((mutations) => {
    mutations.forEach((mutation) => {
      if (mutation.type === 'childList') {
        mutation.addedNodes.forEach((node) => {
          if (node.tagName === 'HEAD') {
            document.head.appendChild(createImportMapScript());
            document.head.appendChild(createIpcScript());
            observer.disconnect();
          }
        });
      }
    });
  })

  observer.observe(document.documentElement, {
    childList: true,
  });
})()