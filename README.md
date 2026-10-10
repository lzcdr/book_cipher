# book_cipher
===========

A console utility for encrypting and decrypting arbitrary data using a
"book" key - any text file available at a URL.

# Idea
----

The long key is the raw byte content of the file at the given URL. It is
never destroyed and can be re-downloaded at any time. The short key is
an offset in bytes into that file. Encryption is byte-wise addition
modulo 256 with a cyclic keystream starting at the given offset.

# Requirements
------------

- Rust (edition 2021)
- Network access: the utility does not work without reaching the URL

# Build
-----

```bash
    cargo build --release
```

Binary: target/release/book_cipher

# Usage
-----

## Encryption:

```bash
    echo -n "secret message" | book_cipher --url <URL> --key <OFFSET>
```

Input: arbitrary bytes (usually text).
Output: base64 string on stdout.

Example:

```bash
    $ echo -n "hello" | book_cipher --url https://example.com/key.txt --key 42
    aGVsbG8=
```

## Decryption:

```bash
    echo -n "aGVsbG8=" | book_cipher --url <URL> --key <OFFSET> -r
```

Input: base64 string (whitespace and newlines are ignored).
Output: original bytes on stdout.

Example:

```bash
    $ echo -n "aGVsbG8=" | book_cipher --url https://example.com/key.txt --key 42 -r
    hello
```

## Interactive input from the keyboard

Run the utility without redirecting stdin. Type or paste your data, then
signal end-of-input:

Linux / macOS: press Ctrl+D

Windows: press Ctrl+Z, then Enter

### Example (encryption):

```bash
$ book_cipher --url https://example.com/key.txt --key 42
hello world
aGVsbG8gd29ybGQ=
```

### Example (decryption):

```bash
$ book_cipher --url https://example.com/key.txt --key 42 -r
aGVsbG8gd29ybGQ=
hello world
```

Note: if you use a pipeline such as "echo ... | book_cipher", the echo
adds a trailing newline. When typing interactively, the newline you press
at the end of the last line is also included in the input. If you need
to control this exactly, use a file or printf instead.

### From a file

Redirect stdin from a file and stdout to a file:

```bash
book_cipher --url <URL> --key <OFFSET> < message.txt > cipher.b64
```

Decrypt back:

```bash
book_cipher --url <URL> --key <OFFSET> -r < cipher.b64 > message.txt
```

No shell quoting, no echo, no trailing newline issues. This is the
recommended way for cold storage: you encrypt a file, store cipher.b64,
and later decrypt it back into a file.

### From another program

```bash
some_program | book_cipher --url <URL> --key <OFFSET>
```

### Via a here-document

Useful for short messages written directly on the command line:

```bash
book_cipher --url <URL> --key <OFFSET> <<'EOF'
hello world
EOF
```

Everything between the two EOF markers is fed to stdin.

### Via printf

"printf" does not add a trailing newline, unlike "echo":

```bash
printf 'hello' | book_cipher --url <URL> --key <OFFSET>
```

### Via a here-string

Works in bash and compatible shells:

```bash
book_cipher --url <URL> --key <OFFSET> <<< "hello world"
```

# Parameters
----------

--url <URL>
    Required. URL of the key text. The file is read as raw bytes,
    without decoding into UTF-8.

--key <N>
    Required. Offset in bytes into the key text. Normalized as
    N mod len(key), so any number is valid.

-r, --reverse
    Decryption mode.

# Specification
-------------

1. The key text is downloaded from the URL as a sequence of bytes
   (not decoded as a string). Any file works: text, binary, anything.

2. The offset sets the starting position in the byte stream of the key.

3. The keystream is the key bytes starting at the offset, cycling
   cyclically. If the message is longer than the key, the stream wraps
   around.

4. Arithmetic is byte-wise modulo 256:

       encryption:   C[i] = (M[i] + K[i]) mod 256
       decryption:   M[i] = (C[i] - K[i] + 256) mod 256

5. Ciphertext output format is base64 (standard alphabet, with padding).

6. Input/output:

       encryption:   stdin  - message bytes,  stdout - base64
       decryption:   stdin  - base64,         stdout - message bytes

# Reversibility
-------------

Addition modulo 256 is reversible. The keystream is deterministic: the
same pair (URL, offset) always produces the same byte sequence.
Therefore decryption always restores the original message, provided the
key text at the URL has not changed.

# Limitations
-----------

- Does not work without network access. The key text is not cached.

- No integrity check. If the file at the URL changes, decryption
  produces garbage without warning.

- Cyclic wrap-around. If the message is longer than the key, the
  keystream repeats. This weakens security on long messages.

- Security is limited. If an adversary knows the URL and the rule, they
  can brute-force the offset. Real secrecy rests on the secrecy of the
  URL and/or the offset.

# Full cycle example
------------------

```bash
    KEY_URL=https://lib.ru/DETEKTIWY/HEMMET/key.txt
    KEY_OFFSET=12345

    echo -n "meeting at 18:00 by the bridge" \
        | book_cipher --url $KEY_URL --key $KEY_OFFSET \
        > cipher.b64

    cat cipher.b64
    <base64 string>

    cat cipher.b64 \
        | book_cipher --url $KEY_URL --key $KEY_OFFSET -r
    meeting at 18:00 by the bridge
```

# License
-------

MIT
