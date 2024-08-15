window.__ = (() => {
  class InvokeError extends Error {
    constructor(message) {
      super(message);
      this.name = 'InvokeError';
    }
  }

  const listenersMap = new Map()

  function on(event, callback) {
    const listeners = listenersMap.get(event) ?? new Set();

    listeners.add(callback);

    listenersMap.set(event, listeners);

    return () => off(event, callback);
  }

  function off(event, callback) {
    listenersMap.get(event)?.delete(callback);
  }

  function dispatch(event, data) {
    listenersMap.get(event)?.forEach(callback => callback(data));
  }

  function createInvokeRequest(method, ...params) {
    let body = null;
    const headers = new Headers({
      'X-Window-Id': window.ID
    });

    if (params.length === 1 && params[0] instanceof ArrayBuffer) {
      headers.set('Content-Type', 'application/octet-stream');
      body = params[0];
    } else {
      headers.set('Content-Type', 'application/json');
      body = JSON.stringify(params);
    }

    return {
      method: 'POST',
      url: `ipc://invoke/${method}`,
      headers,
      body,
    }
  }

  function invokeAsync(name, ...params) {
    const { method, url, headers, body } = createInvokeRequest(name, ...params);

    return fetch(url, { method, headers, body }).then(async (response) => {
      const resultType = response.headers.get('X-Invoke-Result');

      if (resultType === 'Err') {
        if (response.headers.get('Content-Type') === 'application/json') {
          const json = await response.json();
          throw new InvokeError(json);
        } else {
          console.warn('Error response is not JSON', response);
          throw new InvokeError(await response.text());
        }
      }

      if (resultType === 'Ok') {
        if (response.headers.get('Content-Type') === 'application/json') {
          return response.json();
        }

        return response.arrayBuffer()
      }

      throw new Error('Invalid Invoke Result');
    });
  }

  function invokeSync(name, ...params) {
    const { method, url, headers, body } = createInvokeRequest(name, ...params);

    const xhr = new XMLHttpRequest();

    xhr.responseType = 'arraybuffer';

    xhr.open(method, url, false);

    for (const [key, value] of headers.entries()) {
      xhr.setRequestHeader(key, value);
    }

    xhr.send(body);

    const resultType = xhr.getResponseHeader('X-Invoke-Result');

    if (resultType === 'Err') {
      if (xhr.getResponseHeader('Content-Type') === 'application/json') {
        throw new InvokeError(JSON.parse(xhr.responseText));
      } else {
        console.warn('Error response is not JSON', xhr);
        throw new InvokeError(xhr.responseText);
      }
    }

    if (resultType === 'Ok') {
      if (xhr.getResponseHeader('Content-Type') === 'application/json') {
        return JSON.parse(
          new TextDecoder().decode(new Uint8Array(xhr.response))
        )
      }

      return xhr.response;
    }

    throw new Error('Invalid Invoke Result');
  }

  return {
    on,
    off,
    invokeAsync,
    invokeSync,
    dispatch,
  }
})()
