# Readme

Build the rust code using `wasm-pack build --target nodejs`, then delete the release folder so we don't upload that to aws.
Can test locally by just running `node index.mjs`

Standard lambda workflow:

- `zip -r ../ticketlambda.zip *`
- First time only: `aws lambda create-function --function-name ticketlambda --runtime nodejs20.x --role arn:aws:iam::506625725958:role/Radical-Lambda --zip-file fileb://./ticketlambda.zip --handler index.handler`, replacing the role with whatever the role on your account is.
- Subsequent times: `aws lambda update-function-code --function-name ticketlambda --zip-file fileb://./ticketlambda.zip`

# Notes

What did we have to do to get this working:

- Make sure that we're running JS that calls compiled web assembly on both AWS and Cloudflare. Running compiled rust code on AWS is much faster than running web assembly on Cloudflare, which makes sense. Now we're doing an apples-to-apples comparison.
- Restructure the function so that the Rust code _only_ does the CPU heavy tasks. All interaction with Dynamo/Cache is handled in Javascript, because compiling the AWS-sdk for Rust to wasm doesn't work. We did the same for Cloudflare (making Rust only do the compute tasks while JS talks to cache etc), and thankfully that doesn't change the runtime of the function at all, so this _seems_ fine.
- Now we know that the function on AWS and the function on Cloudflare are absolutely doing the same thing, but I worry that there's a very obvious, much faster version of the function that you could run on AWS (the rust code directly)
