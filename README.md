# 🔄 SQS Navbat

## Description

**SQS Navbat** is a local development tool that emulates the AWS SQS API's interface. This allows you to test your SQS integrations without connecting to AWS, which is useful for offline development and testing. It's designed to behave as much as possible like AWS SQS, with the ability to send and get messages, among other functions.

## Features

- Emulates the SQS CreateQueue, SendMessage, ReceiveMessage, ListQueues, DeleteMessage, ChangeMessageVisibility, GetQueueUrl, GetQueueAttributes, SetQueueAttributes
- Error handling similar to the AWS SQS API.

## Installation

```bash
$ git clone git@github.com:oneslash/sqs-navbat.git
$ cd sqs-navbat
$ cargo build
```

## Usage

Start the server:

```bash
$ cargo install sqlx-cli
$ cargo sqlx prepare --database-url sqlite//database.db
$ cargo run
```

Parameters:

- `bind_address` (Default: `"127.0.0.1"`): Defines the IP at which the server will be running. You can modify this value according to your needs.

- `port` (Default: `"9090"`): This is the port number on which the server will listen for requests. If you have another service running on the default port, you may want to change this.
- `db_url` (Default: `"sqlite://database.db"`): DB URL for the Sqlite, currently only SQLite is supported.
- `host_name` (Default: http://localhost:9090) - This will be used for the queue URL creation.

```bash
$ ./s3-chelak --bind_address "0.0.0.0" --port "9090" --db_url "sqlite://database.db" 
```

## API's implemented

| AWS S3 API Name                                              |    Implemented     |
| ------------------------------------------------------------ | :----------------: |
| [AddPermission](https://docs.aws.amazon.com/AWSSimpleQueueService/latest/APIReference/API_AddPermission.html) |        :x:         |
| [CancelMessageMoveTask](https://docs.aws.amazon.com/AWSSimpleQueueService/latest/APIReference/API_CancelMessageMoveTask.html) |        :x:         |
| [ChangeMessageVisibility](https://docs.aws.amazon.com/AWSSimpleQueueService/latest/APIReference/API_ChangeMessageVisibility.html) | :white_check_mark: |
| [ChangeMessageVisibilityBatch](https://docs.aws.amazon.com/AWSSimpleQueueService/latest/APIReference/API_ChangeMessageVisibilityBatch.html) |        :x:         |
| [CreateQueue](https://docs.aws.amazon.com/AWSSimpleQueueService/latest/APIReference/API_CreateQueue.html) | :white_check_mark: |
| [DeleteMessage](https://docs.aws.amazon.com/AWSSimpleQueueService/latest/APIReference/API_DeleteMessage.html) | :white_check_mark: |
| [DeleteMessageBatch](https://docs.aws.amazon.com/AWSSimpleQueueService/latest/APIReference/API_DeleteMessageBatch.html) |        :x:         |
| [DeleteQueue](https://docs.aws.amazon.com/AWSSimpleQueueService/latest/APIReference/API_DeleteQueue.html) |        :x:         |
| [GetQueueAttributes](https://docs.aws.amazon.com/AWSSimpleQueueService/latest/APIReference/API_GetQueueAttributes.html) | :white_check_mark: |
| [GetQueueUrl](https://docs.aws.amazon.com/AWSSimpleQueueService/latest/APIReference/API_GetQueueUrl.html) | :white_check_mark: |
| [ListDeadLetterSourceQueues](https://docs.aws.amazon.com/AWSSimpleQueueService/latest/APIReference/API_ListDeadLetterSourceQueues.html) |        :x:         |
| [ListMessageMoveTasks](https://docs.aws.amazon.com/AWSSimpleQueueService/latest/APIReference/API_ListMessageMoveTasks.html) |        :x:         |
| [ListQueues](https://docs.aws.amazon.com/AWSSimpleQueueService/latest/APIReference/API_ListQueues.html) | :white_check_mark: |
| [ListQueueTags](https://docs.aws.amazon.com/AWSSimpleQueueService/latest/APIReference/API_ListQueueTags.html) |        :x:         |
| [PurgeQueue](https://docs.aws.amazon.com/AWSSimpleQueueService/latest/APIReference/API_PurgeQueue.html) |        :x:         |
| [ReceiveMessage](https://docs.aws.amazon.com/AWSSimpleQueueService/latest/APIReference/API_ReceiveMessage.html) | :white_check_mark: |
| [RemovePermission](https://docs.aws.amazon.com/AWSSimpleQueueService/latest/APIReference/API_RemovePermission.html) |        :x:         |
| [SendMessage](https://docs.aws.amazon.com/AWSSimpleQueueService/latest/APIReference/API_SendMessage.html) | :white_check_mark: |
| [SendMessageBatch](https://docs.aws.amazon.com/AWSSimpleQueueService/latest/APIReference/API_SendMessageBatch.html) |        :x:         |
| [SetQueueAttributes](https://docs.aws.amazon.com/AWSSimpleQueueService/latest/APIReference/API_SetQueueAttributes.html) | :white_check_mark: |
| [StartMessageMoveTask](https://docs.aws.amazon.com/AWSSimpleQueueService/latest/APIReference/API_StartMessageMoveTask.html) |        :x:         |
| [TagQueue](https://docs.aws.amazon.com/AWSSimpleQueueService/latest/APIReference/API_TagQueue.html) |        :x:         |
| [UntagQueue](https://docs.aws.amazon.com/AWSSimpleQueueService/latest/APIReference/API_UntagQueue.html) |        :x:         |

## License

This project is licensed under the MIT License. See the [LICENSE.md](https://chat.openai.com/LICENSE.md) file for details.

## Acknowledgments

- AWS for its comprehensive and well-documented S3 API.