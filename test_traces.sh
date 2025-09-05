#!/bin/bash

echo "🚀 Testing Complex OpenTelemetry Traces"
echo "======================================="
echo
echo "This script generates various trace patterns to explore in Jaeger."
echo "Watch for multi-level spans and different trace shapes!"
echo
sleep 2

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}1️⃣  BATCH OPERATION - Creates nested spans for each item${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo "Creating 5 todos in a single batch request..."
echo "Expected spans: validate_request → create_batch → batch_item_0..4 → database operations"
echo

curl -X POST http://127.0.0.1:3000/todos/batch \
  -H "Content-Type: application/json" \
  -d '{
    "todos": [
      {"title": "Learn Rust ownership", "description": "Master the borrow checker"},
      {"title": "Study async/await", "description": "Understand tokio runtime"},
      {"title": "Implement microservice", "description": "Build with Axum"},
      {"title": "Add observability", "description": "OpenTelemetry tracing"},
      {"title": "Deploy to production", "description": "Use Docker and K8s"}
    ]
  }' 2>/dev/null | jq '.created[] | {id, title}' 2>/dev/null || echo "Batch creation completed"

sleep 1
echo

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}2️⃣  SINGLE TODO WITH NOTIFICATIONS - Shows external service spans${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo "Creating a todo that triggers webhook and email notifications..."
echo "Expected spans: validate_request → create_todo → database.INSERT + send_notifications (webhook + email)"
echo

TODO_ID=$(curl -s -X POST http://127.0.0.1:3000/todos \
  -H "Content-Type: application/json" \
  -d '{"title": "Important: Review PR #42", "description": "Critical security fix - triggers notifications"}' 2>/dev/null | jq -r '.id' 2>/dev/null)

if [ ! -z "$TODO_ID" ] && [ "$TODO_ID" != "null" ]; then
    echo -e "${YELLOW}Created todo with ID: $TODO_ID${NC}"
else
    echo "Todo created (notifications may have been triggered)"
fi

sleep 1
echo

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}3️⃣  LIST OPERATION - Database query with simulated latency${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo "Listing all todos (shows database SELECT span)..."
echo "Expected spans: validate_request → list_todos → database.SELECT_ALL"
echo

TODO_COUNT=$(curl -s http://127.0.0.1:3000/todos 2>/dev/null | jq '. | length' 2>/dev/null)
echo -e "${YELLOW}Total todos in system: ${TODO_COUNT:-unknown}${NC}"

sleep 1
echo

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}4️⃣  UPDATE WITH COMPLETION - Triggers analytics event${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

if [ ! -z "$TODO_ID" ] && [ "$TODO_ID" != "null" ]; then
    echo "Completing todo $TODO_ID (triggers completion notification)..."
    echo "Expected spans: validate_request → update_todo → database.SELECT + database.UPDATE + analytics_event"
    echo
    
    curl -X PUT http://127.0.0.1:3000/todos/$TODO_ID \
      -H "Content-Type: application/json" \
      -d '{"completed": true, "title": "COMPLETED: Review PR #42"}' 2>/dev/null | jq '{id, title, completed}' 2>/dev/null || echo "Todo updated"
else
    echo "Skipping update (no valid TODO_ID)"
fi

sleep 1
echo

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}5️⃣  PARALLEL OPERATIONS - Multiple requests at once${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo "Sending 3 requests simultaneously to see concurrent traces..."
echo

# Run three operations in parallel
(curl -s -X POST http://127.0.0.1:3000/todos \
  -H "Content-Type: application/json" \
  -d '{"title": "Parallel Task 1"}' >/dev/null 2>&1 && echo "  ✓ Created Parallel Task 1") &

(curl -s -X POST http://127.0.0.1:3000/todos \
  -H "Content-Type: application/json" \
  -d '{"title": "Parallel Task 2"}' >/dev/null 2>&1 && echo "  ✓ Created Parallel Task 2") &

(curl -s -X POST http://127.0.0.1:3000/todos \
  -H "Content-Type: application/json" \
  -d '{"title": "Parallel Task 3"}' >/dev/null 2>&1 && echo "  ✓ Created Parallel Task 3") &

wait
sleep 1
echo

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}6️⃣  GET BY ID - Single item fetch with potential 404${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

if [ ! -z "$TODO_ID" ] && [ "$TODO_ID" != "null" ]; then
    echo "Fetching specific todo by ID..."
    echo "Expected spans: validate_request → get_todo → database.SELECT"
    echo
    curl -s http://127.0.0.1:3000/todos/$TODO_ID 2>/dev/null | jq '{id, title, completed}' 2>/dev/null || echo "Todo fetched"
else
    echo "Skipping fetch (no valid TODO_ID)"
fi

sleep 1
echo

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}7️⃣  DELETE COMPLETED - Bulk delete operation${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo "Deleting all completed todos..."
echo "Expected spans: validate_request → delete_completed → database.DELETE_COMPLETED"
echo

DELETED=$(curl -s -X DELETE http://127.0.0.1:3000/todos/completed 2>/dev/null | jq -r '.deleted_count' 2>/dev/null)
echo -e "${YELLOW}Deleted ${DELETED:-0} completed todos${NC}"

sleep 1
echo

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}8️⃣  ERROR SCENARIO - 404 Not Found${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo "Attempting to fetch non-existent todo..."
echo "Expected spans: validate_request → get_todo → database.SELECT (with error)"
echo

curl -s http://127.0.0.1:3000/todos/00000000-0000-0000-0000-000000000000 -w "\nHTTP Status: %{http_code}\n" 2>/dev/null

sleep 1
echo

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}9️⃣  HEALTH CHECK - Database connectivity check${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo "Checking API health..."
echo "Expected spans: validate_request → health_check → database.SELECT_ALL"
echo

curl -s http://127.0.0.1:3000/health 2>/dev/null | jq '.' 2>/dev/null || echo "Health check completed"

echo
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}✅ Test scenarios completed!${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo
echo "🔍 Now check Jaeger at http://localhost:16686"
echo
echo "📊 What to look for in Jaeger:"
echo "   1. Service: 'todo-api'"
echo "   2. Operations to explore:"
echo "      - create_batch (most complex with nested spans)"
echo "      - create_todo (with notification spans)"
echo "      - update_todo (with completion notification)"
echo "      - list_todos (database query)"
echo "      - validate_request (appears in all traces)"
echo
echo "🎯 Interesting trace patterns:"
echo "   - Batch operations show iteration through items"
echo "   - Notification calls show external service simulation"
echo "   - Database operations show simulated latency"
echo "   - Error traces show where failures occur"
echo "   - Parallel requests show concurrent trace timelines"
echo
echo "💡 Tips:"
echo "   - Click on a trace to see the full span hierarchy"
echo "   - Look at span attributes for context (IDs, counts, etc.)"
echo "   - Compare timings between different operations"
echo "   - Filter by operation name to focus on specific patterns"
echo "   - Use the timeline view to understand parallelism"