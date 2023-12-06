import logging
import boto3
from boto3.dynamodb.conditions import Key
from botocore.exceptions import ClientError

logger = logging.getLogger(__name__)


class RadicalTicket:
  """Encapsulates an Amazon DynamoDB table of ticket data."""

  def __init__(self, dyn_resource):
    """
    :param dyn_resource: A Boto3 DynamoDB resource.
    """
    self.dyn_resource = dyn_resource
    # The table variable is set during the scenario in the call to
    # 'exists' if the table exists. Otherwise, it is set by 'create_table'.
    self.table = None

  # snippet-end:[python.example_code.dynamodb.helper.Movies.class_decl]

  # snippet-start:[python.example_code.dynamodb.DescribeTable]
  def exists(self, table_name):
    """
    Determines whether a table exists. As a side effect, stores the table in
    a member variable.

    :param table_name: The name of the table to check.
    :return: True when the table exists; otherwise, False.
    """
    try:
      table = self.dyn_resource.Table(table_name)
      table.load()
      exists = True
    except ClientError as err:
      if err.response["Error"]["Code"] == "ResourceNotFoundException":
          exists = False
      else:
          logger.error(
              "Couldn't check for existence of %s. Here's why: %s: %s",
              table_name,
              err.response["Error"]["Code"],
              err.response["Error"]["Message"],
          )
          raise
    else:
      self.table = table
    return exists
  
  def create_n_tickets(self, n):
    try:
      with self.table.batch_writer() as writer:
        for i in range(n):
          key = f"ticket-{i}"
          id = key
          version = 0
          value = {
            "id": i,
            "taken": False,
            "res_email": None,
            "res_name": None,
            "res_card": None,
          }
          writer.put_item(Item={
            "Key": key,
            "ID": id,
            "Version": version,
            "Value": value,
          })
    except ClientError as err:
      logger.error(
        "Couldn't load data into table %s. Here's why: %s: %s",
        self.table.name,
        err.response["Error"]["Code"],
        err.response["Error"]["Message"],
      )
      raise


# given dynamo table name, dynamo resource, number of tickets to create n
def run_scenario(n):
  table_name = "Radical-Ticket"
  dyn_resource = boto3.resource("dynamodb")
  logging.basicConfig(level=logging.INFO, format="%(levelname)s: %(message)s")

  # print("-" * 88)
  # print("Welcome to the Amazon DynamoDB getting started demo.")
  # print("-" * 88)

  radical_ticket = RadicalTicket(dyn_resource)
  exists = radical_ticket.exists(table_name)
  if not exists:
    print("ERROR: TABLE DOES NOT EXIST")

  radical_ticket.create_n_tickets(n)
  

if __name__ == "__main__":
  try:
    run_scenario(10)
  except Exception as e:
    print(f"Something went wrong with the setup! Here's what: {e}")
