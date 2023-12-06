import { greet, anti_fraud } from './pkg/ticketlambda.js';
import { DynamoDB } from "@aws-sdk/client-dynamodb"
import { DynamoDBDocumentClient, GetCommand, PutCommand } from "@aws-sdk/lib-dynamodb"
import { performance, PerformanceObserver } from "perf_hooks"

const getKey = async (docClient, key) => {
  const command = new GetCommand({
    TableName: "Radical-Ticket",
    Key: {
      Key: key,
    },
  });
  const resp = await docClient.send(command)
  return resp.Item
}

const putKey = async (docClient, key, value) => {
	const command = new PutCommand({
		TableName: "Radical-Ticket",
		Item: value,
	})
	const resp = await docClient.send(command);
	return resp
}

export const handler = async(event, ctx) => {
	const jsonBody = JSON.parse(event.body)
	let { id, taken, res_email, res_name, res_card } = jsonBody
	let ticketKey = `ticket-${id}`;

  const client = new DynamoDB({ region: "us-east-2" });
  const docClient = DynamoDBDocumentClient.from(client)

	// console.log("Fetching ticket with key", ticketKey)
	// let item = await getKey(docClient, ticketKey)
	// console.log("Got item", item)
	// console.log(item)

	let item = {
		Version: 19,
		ID: 'ticket-0',
		Value: {
			id: 0,
			res_name: 'Test Name0',
			res_email: 'test_0@test.com',
			res_card: '0xxxx1234',
			taken: true
		},
		Key: 'ticket-0'
	}

	// Do the anti fraud stuff
	let antiFraudCheck = true
	antiFraudCheck = anti_fraud(res_email, res_name, res_card)
	console.log("Result of calling anti fraud", antiFraudCheck)
	if (antiFraudCheck) {
		console.log("Updating item to version", item.Version + 1)
		let new_item = {
			Key: ticketKey,
			ID: ticketKey,
			Version: item.Version + 1,
			Value: {
				id: id,
				taken: true,
				res_email: res_email,
				res_name: res_name,
				res_card: res_card
			}
		}
		// let resp = await putKey(docClient, ticketKey, new_item);
		// console.log(resp)
	}

	greet("hello")
	let respBody = { done: true }
	return {
		statusCode: 200,
		body: JSON.stringify(respBody)
	}
}

const perfObserver = new PerformanceObserver((items) => {
  items.getEntries().forEach((entry) => {
    console.log(entry)
  })
})

perfObserver.observe({ entryTypes: ["measure"], buffer: true })

for (let i = 0; i < 1; i++) {
	performance.mark("start-req")
	let result = await handler({
		body: "{\"id\": 0, \"taken\": true, \"res_email\": \"xx@x.com\", \"res_name\": \"yy\", \"res_card\": \"zz\"}"
	}, undefined)
	performance.mark("end-req")
	performance.measure("request-time", "start-req", "end-req")
}
