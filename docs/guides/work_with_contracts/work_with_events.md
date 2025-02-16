# Work with Events

How to work with contract events

* [About events](work\_with\_events.md#about-events)
* [Query/subscribe to events](work\_with\_events.md#querysubscribe-to-events)
* [Query/Subscribe with SDK](work\_with\_events.md#querysubscribe-with-sdk)
* [Low level syntax](work\_with\_events.md#low-level-syntax)
  * [Query](work\_with\_events.md#query)
  * [Subscribe](work\_with\_events.md#subscribe)
  * [Decode](work\_with\_events.md#decode)

## About events

When contract emits an event, you can fetch it from blockchain or you can subscribe to it.

Events in blockchain are external outbound messages. In GraphQL API their `msg_type` is 2.

## Query/subscribe to events

You can fetch events of you contract with this filter from graphql. Try it out in playground [https://eri01.main.everos.dev/graphql](https://eri01.main.everos.dev/graphql):

```graphql
query{
messages(
      filter:{ 
      src:{
        eq:"-1:67f4bf95722e1bd6df845fca7991e5e7128ce4a6d25f6d4ef027d139a11a7964"
      }
      msg_type:{
        eq:2
      }
    }
)
{
   id
 body
}
}
```

Or subscribe to them:

```graphql
subscription{
messages(
      filter:{ 
      src:{
        eq:"-1:67f4bf95722e1bd6df845fca7991e5e7128ce4a6d25f6d4ef027d139a11a7964"
      }
      msg_type:{
        eq:2
      }
    }
)
{
   id
  body
}
}
```

## Query/Subscribe with SDK

Let's assume our contract code is this:

```solidity
pragma ton-solidity >= 0.38.2;
pragma AbiHeader expire;


contract HelloEvents {
// Event is an external message generated by the contract functions.
// Here we will emit this external outbound message (event)
// every time we have changed the hello text.
event TextUpdated(string text, uint32 time);

// Instance variable storing some user data.
string helloText;

// Instance variable storing the time of `constructor` call or `setHelloText` function call.
uint32 textUpdateTime;

// Constructor sets instance variables.
// All contracts need to call `tvm.accept()` for deploying.
constructor(string text) public {
    tvm.accept();
    helloText = text;
    textUpdateTime = now;
}

// Function `setHelloText` updates instance variables
// `helloText` and `textUpdateTime` 
// and emits `TextUpdated` event.
function setHelloText(string text) external returns (string oldText) {
    require(msg.pubkey() == tvm.pubkey(), 100);
    tvm.accept();
    string saveText = helloText;
    helloText = text;
    textUpdateTime = now;
    emit TextUpdated(helloText, textUpdateTime);
    return saveText;
}

// Function returns value of instance variable `helloText`.
// This function is a get method (it does not change state and has no `accept` function)
// so it can be called only on local TVM.
function getHelloText() public view returns (string text) {
    return helloText;
}

// Function returns value of instance variable `textUpdateTime`.
// This function is a get method (it does not change state and has no `accept` function)
// so it can be called only on local TVM.
function getTextUpdateTime() public view returns (uint32 time) {
    return textUpdateTime;
}
}
```

We see that we have 1 event `TextUpdated(helloText, textUpdateTime)`.

## Low level syntax

See the full sample here [https://github.com/tonlabs/sdk-samples/tree/master/core-examples/node-js/listen-and-decode](https://github.com/tonlabs/sdk-samples/tree/master/core-examples/node-js/listen-and-decode)

### Query

```javascript
result = (await client.net.query_collection({
        collection: "messages",
        filter: {
            src: {
                eq: "-1:67f4bf95722e1bd6df845fca7991e5e7128ce4a6d25f6d4ef027d139a11a7964",
            },
            msg_type:{ eq:2 }
        },
        result: "boc",
})).result;
```

### Subscribe

To subscribe to new events do this. Don't forget to specify your own callback.

```javascript
const messageSubscription = await TonClient.default.net.subscribe_collection({
    collection: "messages",
    filter: {
        dst: { eq: address },
        msg_type:{ eq: 2 }
    },
    result: "boc"
}, <callback function>
});
```

### Decode

```javascript
const decoded = (await TonClient.default.abi.decode_message({
                    abi: abiContract(HelloEventsContract.abi),
                    message: params.result.boc,
                }));
switch (decoded.body_type) {
                case MessageBodyType.Input:
                    console.log(`External inbound message, function "${decoded.name}", parameters: `, JSON.stringify(decoded.value));
                    break;
                case MessageBodyType.Output:
                    console.log(`External outbound message, function "${decoded.name}", result`, JSON.stringify(decoded.value));
                    break;
                case MessageBodyType.Event:
                    console.log(`External outbound message, event "${decoded.name}", parameters`, JSON.stringify(decoded.value));
                    break;
                }
```

Check out [AppKit documentation](https://docs.everos.dev/appkit-js/guides/work\_with\_events) for this use case.
