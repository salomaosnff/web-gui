window.__ = (() => {
  const listenersMap = new Map()

  function on(event, callback) {
    const listeners = listenersMap.get(event) ?? new Set();

    listeners.add(callback);

    listenersMap.set(event, listeners);

    return () => off(event, callback);
  }

  function off(event, callback) {
    const listeners = listenersMap.get(event);

    if (listeners) {
      listeners.delete(callback);
    }
  }

  function dispatch(event, data) {
    const listeners = listenersMap.get(event);

    if (listeners) {
      listeners.forEach(callback => callback(data));
    }
  }

  function invokeAsync(method, ...params) {
    return fetch(`ipc://invoke/${method}`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'X-Window-Id': window.ID
      },
      body: JSON.stringify(params)
    }).then((res) => res.json()).then(json => {
      if ('Err' in json) {
        throw new Error(json.Err);
      }

      if ('Ok' in json) {
        return json.Ok;
      }

      return json;
    });
  }

  function invokeSync(method, ...params) {
    const xhr = new XMLHttpRequest();

    xhr.open('POST', `ipc://invoke/${method}`, false);

    xhr.setRequestHeader('Content-Type', 'application/json');
    xhr.setRequestHeader('X-Window-Id', window.ID);

    xhr.send(JSON.stringify(params));

    const json = JSON.parse(xhr.responseText);

    if ('Err' in json) {
      throw new Error(json.Err);
    }

    if ('Ok' in json) {
      return json.Ok;
    }
  }

  return {
    on,
    off,
    invokeAsync,
    invokeSync,
    dispatch,
  }
})()
