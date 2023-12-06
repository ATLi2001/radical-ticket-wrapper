import requests
import time
import pandas as pd
import argparse

import dynamo_setup

class TicketBenchmark:
  def __init__(self, target: str, backup_url: str, consistency_check_url: str) -> None:
    # base target url
    self.target = target
    # keep request session alive
    self.session = requests.Session()
    # lambda backup url
    self.backup_url = backup_url
    # consistency check url
    self.consistency_check_url = consistency_check_url

  # populate the cache with n tickets
  def populate_tickets(self, n) -> None:
    url = self.target + "/populate_tickets"
    resp = self.session.post(url, data=f"{n}")
    assert resp.status_code == 200

  # clear the cache
  def clear_cache(self) -> None:
    url = target + "/clear_cache"
    resp = self.session.post(url)
    assert resp.status_code == 200
    print(resp.text)

  # list all available tickets
  def avail_tickets(self) -> None:
    resp = self.session.get(target)
    if resp.status_code == 200:
      print(resp.content)
    else:
      print("avail_tickets error", resp.status_code)

  # get ticket i
  def get_ticket(self, i: int) -> str:
    url = target + f"/get_ticket/{i}"
    resp = self.session.get(url)
    assert resp.status_code == 200
    return resp.text

  # reserve ticket i and return the time it took in ms
  def reserve_ticket(self, i: int) -> float:
    ticket_data = {
      "id": i,
      "taken": True,
      "res_email": f"test_{i}@test.com",
      "res_name": f"Test Name{i}",
      "res_card": f"{i}xxxx1234",
    }

    reqData = {
      "remoteUrl": self.consistency_check_url,
      "backup": self.backup_url,
      "args": ticket_data,
    }

    url = target + "/reserve"
    start = time.perf_counter()
    resp = self.session.post(url, json=reqData)
    end = time.perf_counter()
    if resp.status_code != 200:
      print(f"ERROR: reserve_ticket({i})", resp)
    else:
      print(resp.text, "in", (end-start)*1000)

    # milliseconds
    return (end - start) * 1000


if __name__ == "__main__":
  # Parse in command-line arguments.
  parser = argparse.ArgumentParser(
    prog="ticket-benchmark",
    description="Creates tickets and measures latency of reserving a ticket",
  )
  parser.add_argument("-d", "--dev", action="store_true", help="use the local dev server rather than the Cloudflare deployment")
  args = parser.parse_args()

  # Set target depending on dev vs. prod.
  if args.dev:
    target = "http://localhost:8787"
    env_name = "local"
  else:
    target = "https://ticket-bench-orch.radical-serverless.com"
    env_name = "edge"

  n = 10
  trials = 10

  consistency_check_url = "https://nuamf2bgzlrfj6vubqfzkjv52m0kpefu.lambda-url.us-east-2.on.aws/"
  lambda_url = "https://c54mpf4fcxguxzvatjfjocaabu0kgsuz.lambda-url.us-east-2.on.aws/"

  results = pd.DataFrame(columns=[f"ticket{i}_ms" for i in range(n)])

  ticket_bench = TicketBenchmark(target, lambda_url, consistency_check_url)

  print("initializing dynamo")
  dynamo_setup.run_scenario(n)
  time.sleep(1)

  for t in range(trials):
    print("trial", t)

    ticket_bench.populate_tickets(n)
    time.sleep(1)

    trial_results = []
    for i in range(n):
      print(f"ticket-{i}")
      trial_results.append(ticket_bench.reserve_ticket(i))

    results.loc[len(results)] = trial_results

    ticket_bench.clear_cache()
    time.sleep(1)

  results.to_csv(f"anti_fraud_{env_name}_{n}tickets_{trials}trials.csv")
