package main

import (
	"context"
	"flag"
	log "github.com/sirupsen/logrus"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
	"tableflow/alpha/common/probe"
	"time"
)

var (
	target      = flag.String("TARGET", "", "healthcheck target")
	serviceEnum = flag.Int("SERVICE", 0, "healthcheck service")
	probeEnum   = flag.Int("PROBE", 0, "healthcheck probe")
	methodName  = flag.String("METHOD", "", "healthcheck method name")
)

func main() {
	flag.Parse()

	nodeType := probe.ProbeRequest_NodeType(*serviceEnum)

	ctx, cancelFunc := context.WithTimeout(context.Background(), time.Second*5)
	defer cancelFunc()

	conn, err := grpc.Dial(*target, grpc.WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		log.Fatal(err)
	}
	var resp probe.ProbeResponse
	err = conn.Invoke(ctx, *methodName, &probe.ProbeRequest{
		NodeType:  nodeType,
		ProbeType: probe.ProbeRequest_ProbeType(*probeEnum),
	}, &resp)

	if err != nil {
		log.Fatalf("health check failed: %v", err)
	}
}
