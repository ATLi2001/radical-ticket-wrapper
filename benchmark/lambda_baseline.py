import requests
import time
import pandas as pd
import dynamo_setup


class LambdaBaseline:
  def __init__(self, target: str) -> None:
    # base target url
    self.target = target
    # keep request session alive
    self.session = requests.Session()

  # reset dynamo
  def reset_dynamo(self, n: int) -> None:
    dynamo_setup.run_scenario(n)

  # reserve ticket i and return the time it took in ms
  def reserve_ticket(self, i: int) -> float:
    ticket_data = {
      "id": i,
      "taken": True,
      "res_email": f"test_{i}@test.com",
      "res_name": f"Test Name{i}", 
      "res_card": f"{i}xxxx1234", 
    }

    url = target
    start = time.perf_counter()
    resp = self.session.post(url, json=ticket_data)
    end = time.perf_counter()
    if resp.status_code != 200:
      print(f"ERROR: reserve_ticket({i})", resp)

    # milliseconds
    return (end - start) * 1000


if __name__ == "__main__":
  target = "https://67f42q3sp4gqm7rfgvjngamyra0wrsew.lambda-url.us-east-2.on.aws"

  n = 10
  trials = 10

  results = pd.DataFrame(columns=[f"ticket{i}_ms" for i in range(n)])

  baseline = LambdaBaseline(target)

  for t in range(trials):
    print("trial", t)
    baseline.reset_dynamo(n)
    time.sleep(1)

    trial_results = []
    for i in range(n):
      trial_results.append(baseline.reserve_ticket(i))
    
    results.loc[len(results)] = trial_results
    time.sleep(1)
  
  results.to_csv(f"lambda_baseline_{n}tickets_{trials}trials.csv")
