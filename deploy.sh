npx tsc

echo 'y' | clasp push

clasp deploy

deployments=$(clasp deployments)

latest=$(echo "$deployments" | grep -oP '^[-] \K[A-Za-z0-9-_]+' | head -n 1)

if [ -n "$latest" ]; then
  echo "Web App URL:"
  echo "https://script.google.com/macros/s/$latest/exec"
else
  echo "not found"
  exit 1
fi
