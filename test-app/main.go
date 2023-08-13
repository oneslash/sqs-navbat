package main

import (
	"context"
	"fmt"
	"log"

	"github.com/aws/aws-sdk-go-v2/aws"
	"github.com/aws/aws-sdk-go-v2/config"
	"github.com/aws/aws-sdk-go-v2/service/sqs"
)

func main() {
	resolver := aws.EndpointResolverWithOptionsFunc(func(service, region string, options ...interface{}) (aws.Endpoint, error) {
		if service == sqs.ServiceID {
			return aws.Endpoint{
				URL:           "http://localhost:9090", // Custom endpoint URL
				SigningRegion: "us-west-2",
			}, nil
		}
		return aws.Endpoint{}, &aws.EndpointNotFoundError{}
	})

	cfg, err := config.LoadDefaultConfig(context.TODO(), config.WithEndpointResolverWithOptions(resolver))
	if err != nil {
		log.Fatal(err)
	}

	client := sqs.NewFromConfig(cfg)
	_, err = client.CreateQueue(context.TODO(), &sqs.CreateQueueInput{
		QueueName: aws.String("myqueue"),
	})
	if err != nil {
		log.Fatal(err)
	}

	output, err := client.ListQueues(context.TODO(), &sqs.ListQueuesInput{})
	if err != nil {
		log.Fatal(err)
	}

	client.SendMessage(context.TODO(), &sqs.SendMessageInput{
		MessageBody: aws.String("Hello world!"),
		QueueUrl:    aws.String("http://localhost:9090/queue/myqueue"),
	})
	client.SendMessage(context.TODO(), &sqs.SendMessageInput{
		MessageBody: aws.String("Hello world!"),
		QueueUrl:    aws.String("http://localhost:9090/queue/myqueue"),
	})

	message, err := client.ReceiveMessage(context.TODO(), &sqs.ReceiveMessageInput{
		QueueUrl: aws.String("http://localhost:9090/queue/myqueue"),
	})
	if err != nil {
		log.Fatal(err)
	}
	fmt.Printf("Message: %s", *message.Messages[0].Body)
	fmt.Println("")
	fmt.Printf("Queues:\n, %v", output.QueueUrls)
}
