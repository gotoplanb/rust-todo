#!/bin/bash

echo "🧪 Testing CRUD Operations..."
echo

echo "1. Creating a todo..."
TODO_ID=$(curl -s -X POST http://127.0.0.1:3000/todos \
  -H "Content-Type: application/json" \
  -d '{"title": "Learn OpenTelemetry", "description": "Add tracing to Rust API"}' | jq -r '.id')
echo "Created todo with ID: $TODO_ID"
echo

echo "2. Getting the created todo..."
curl -s http://127.0.0.1:3000/todos/$TODO_ID | jq
echo

echo "3. Updating the todo..."
curl -s -X PUT http://127.0.0.1:3000/todos/$TODO_ID \
  -H "Content-Type: application/json" \
  -d '{"completed": true, "title": "Learn OpenTelemetry (Done!)"}' | jq
echo

echo "4. Listing all todos..."
curl -s http://127.0.0.1:3000/todos | jq
echo

echo "5. Deleting the todo..."
curl -s -X DELETE http://127.0.0.1:3000/todos/$TODO_ID -w "\nStatus: %{http_code}\n"
echo

echo "6. Verifying deletion (should return 404)..."
curl -s http://127.0.0.1:3000/todos/$TODO_ID -w "\nStatus: %{http_code}\n"