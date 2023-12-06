# Readme

Build the rust code using `wasm-pack build --target nodejs`, then delete the release folder so we don't upload that to aws.
Can test locally by just running `node index.mjs`

Standard lambda workflow:

- `zip -r ../ticketlambda.zip *`
- First time only: `aws lambda create-function --function-name ticketlambda --runtime nodejs20.x --role arn:aws:iam::506625725958:role/Radical-Lambda --zip-file fileb://./ticketlambda.zip --handler index.handler`
- Subsequent times: `aws lambda update-function-code --function-name ticketlambda --zip-file fileb://./ticketlambda.zip`
