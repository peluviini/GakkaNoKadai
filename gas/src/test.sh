
WEBHOOK_URL="https://discord.com/api/webhooks/1392685132707528736/gjHwgfq3JttlRXPBOBvN0nqJNToUlo3RwN1yIiuz0GTWXuP8ir7Iq7CA3Ax1XhFVODBt"

MESSAGE_CONTENT="Hello World!"

REQUEST_BODY="{\"content\":\"$MESSAGE_CONTENT\"}"

curl -X POST -H "Content-Type: application/json" -d "$REQUEST_BODY" "$WEBHOOK_URL"
