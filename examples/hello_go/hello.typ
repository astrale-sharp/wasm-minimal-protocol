#{
  let p = plugin("./hello.wasm")

  assert.eq(str(p.hello()), "Hello from wasm!!!")
  assert.eq(str(p.double_it(bytes("abc"))), "abcabc")
  assert.eq(str(p.concatenate(bytes("hello"), bytes("world"))), "hello*world")
  assert.eq(str(p.shuffle(bytes("s1"), bytes("s2"), bytes("s3"))), "s3-s1-s2")
  assert.eq(str(p.returns_ok()), "This is an `Ok`")
  assert.eq(str(p.set_to_a(bytes("xxxyyz"))), "aaaaaa")
  // p.will_panic()  // Fails compilation
  // p.returns_err() // Fails compilation with an error message

  let encoded = cbor.encode((x: 1, y: 2.0))
  let decoded = cbor(p.complex_data(encoded))
  assert.eq(decoded, 3.0)
}
