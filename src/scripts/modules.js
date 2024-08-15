
(() => {
  const script = document.createElement('script');

  script.type = 'importmap';

  console.log(get_import_map());

  /**
   * get_import_map() é uma macro.
   */
  script.textContent = JSON.stringify({
    imports: get_import_map()
  });

  const observer = new MutationObserver((mutations) => {
    mutations.forEach((mutation) => {
      if (mutation.type === 'childList') {
        mutation.addedNodes.forEach((node) => {
          if (node.tagName === 'HEAD') {
            document.head.appendChild(script);
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