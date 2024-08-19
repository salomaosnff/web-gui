export class InvokeError extends Error {
  constructor(message) {
    super(message);
    this.name = 'InvokeError';
  }
}

const listenersMap = new Map()

export function on(event, callback) {
  const listeners = listenersMap.get(event) ?? new Set();

  listeners.add(callback);

  listenersMap.set(event, listeners);

  return () => off(event, callback);
}

export function off(event, callback) {
  listenersMap.get(event)?.delete(callback);
}

window.__dispatch = function dispatch(event, data) {
  listenersMap.get(event)?.forEach(callback => callback(data));
}

export function createInvokeRequest(method, ...params) {
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
    url: window.CUSTOM_PROTOCOL('ipc', `invoke/${method}`),
    headers,
    body,
  }
}

export function invokeAsync(name, ...params) {
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

    console.log(...(response.headers.entries()));

    throw new Error('Invalid Invoke Result');
  });
}

export function invokeSync(name, ...params) {
  const { method, url, headers, body } = createInvokeRequest(name, ...params);

  const xhr = new XMLHttpRequest();

  xhr.open(method, url, false);

  for (const [key, value] of headers.entries()) {
    xhr.setRequestHeader(key, value);
  }

  xhr.send(body);

  const resultType = xhr.getResponseHeader('X-Invoke-Result');

  if (resultType === 'Err') {
    if (xhr.getResponseHeader('Content-Type') === 'application/json') {
      throw new InvokeError(JSON.parse(new TextDecoder().decode(new Uint8Array(xhr.response))));
    } else {
      console.warn('Error response is not JSON', xhr);
      throw new InvokeError(new TextDecoder().decode(new Uint8Array(xhr.response)));
    }
  }
  
  if (resultType === 'Ok') {
    if (xhr.getResponseHeader('Content-Type') === 'application/json') {
      if (typeof xhr.response === 'string') {
        return JSON.parse(xhr.response)
      }
      return JSON.parse(
        new TextDecoder().decode(new Uint8Array(xhr.response))
      )
    }

    return xhr.response;
  }

  throw new Error('Invalid Invoke Result');
}