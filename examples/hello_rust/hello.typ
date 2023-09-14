#{
  let p = plugin("./hello.wasm")

  assert(str(p.hello()) == "Hello from wasm!!!")
  assert(str(p.double_it(bytes("abc"))) == "abcabc")
  assert(str(p.concatenate(bytes("hello"), bytes("world"))) == "hello*world")
  assert(str(p.shuffle(bytes("s1"), bytes("s2"), bytes("s3"))) == "s3-s1-s2")
  assert(str(p.returns_ok()) == "This is an `Ok`")

  // p.will_panic()  // Fails compilation
  // p.returns_err() // Fails compilation with an error message
}


#{
  let p = plugin("./hello-wasi.wasm")

  assert(str(p.hello()) == "Hello from wasm!!!")
  assert(str(p.double_it(bytes("abc"))) == "abcabc")
  assert(str(p.concatenate(bytes("hello"), bytes("world"))) == "hello*world")
  assert(str(p.shuffle(bytes("s1"), bytes("s2"), bytes("s3"))) == "s3-s1-s2")
  assert(str(p.returns_ok()) == "This is an `Ok`")

  // p.will_panic()  // Fails compilation
  // p.returns_err() // Fails compilation with an error message
}
