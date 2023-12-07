import requests
import time
import sys

url = sys.argv[1] + "/direct_invoke"
s = requests.Session()

def populate_tickets(n) -> None:
	url = sys.argv[1] + "/populate_tickets"
	resp = s.post(url, data=f"{n}")
	print("Got response to populate request")
	assert resp.status_code == 200

print("Sending invocations to", url)
if len(sys.argv) > 4:
	print("populating")
	populate_tickets(10)

for i in range(int(sys.argv[2])):
	start = time.perf_counter()
	s.put(url)
	end = time.perf_counter()
	print("Runtime", (end - start)*1000)
