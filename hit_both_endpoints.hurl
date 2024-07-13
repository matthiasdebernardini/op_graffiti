# test_op_return.hurl

# Test GET endpoint
GET http://localhost:9000/get_op_return/hello_world
HTTP/1.1 200
[Asserts]
body == "hello_world"

# Test WRITE endpoint
GET http://localhost:9000/write_op_return/test_data
HTTP/1.1 200
[Asserts]
body == "test_data"

# Test GET endpoint with special characters
GET http://localhost:9000/get_op_return/Hello,%20World!
HTTP/1.1 200
[Asserts]
body == "Hello, World!"

# Test WRITE endpoint with numbers
GET http://localhost:9000/write_op_return/12345
HTTP/1.1 200
[Asserts]
body == "12345"