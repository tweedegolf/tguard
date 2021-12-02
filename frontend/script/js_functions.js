export async function encrypt(message, key, iv) {
  try {
    const keyHash = await crypto.subtle.digest('SHA-256', key);
    const aesKey = await window.crypto.subtle.importKey(
      'raw',
      keyHash,
      'AES-GCM',
      true,
      ['encrypt']
    );
    const ct = await window.crypto.subtle.encrypt(
      {
        name: 'AES-GCM',
        iv,
      },
      aesKey,
      (new TextEncoder()).encode(message),
    );

    return new Uint8Array(ct);
  } catch (e) {
    console.error(e);
    return null;
  }
}

export async function irma(session) {
  const identity = { type: session.attribute_identifier, value: session.attribute_value };

  try {
    const usk = await window.startIrma({
      identity,
      timestamp: session.timestamp,
      maxAge: 300,
      url: 'https://irmacrypt.nl/pkg',
      start: {
        url: (o) => `${o.url}/v1/request`,
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          attribute: identity,
        }),
      },
      state: { serverSentEvents: false },
      mapping: {
        sessionPtr: (r) => JSON.parse(r.qr),
      },
      result: {
        url: (o, { sessionToken }) => `${o.url}/v1/request/${sessionToken}/${o.timestamp.toString()}`,
        parseResponse: (r) => r.json().then((r) => (r.status === 'DONE_VALID' ? r.key : null)),
      },
    });

    return usk;
  } catch (e) {
    console.error(e);
    return null;
  }
}

export async function decrypt(ciphertext, key, iv) {
  try {
    const keyHash = await crypto.subtle.digest('SHA-256', key);
    const aesKey = await window.crypto.subtle.importKey(
      'raw',
      keyHash,
      'AES-GCM',
      true,
      ['decrypt']
    );
    const encoded = await window.crypto.subtle.decrypt(
      {
        name: 'AES-GCM',
        iv,
      },
      aesKey,
      ciphertext
    );

    return (new TextDecoder()).decode(encoded).toString();
  } catch (e) {
    console.error(e);
    return null;
  }
}

export async function decrypt_cfb_hmac(ciphertext, key, iv) {
  try {
    const aesKey = await window.crypto.subtle.importKey(
      'raw',
      key,
      { name: 'AES-CTR', length: 32 * 8 },
      true,
      ['decrypt']
    );
    const encoded = await window.crypto.subtle.decrypt(
      {
        name: 'AES-CTR',
        counter: iv,
        length: 64
      },
      aesKey,
      ciphertext
    );

    return encoded;
  } catch (e) {
    console.error(e);
    return null;
  }
}

