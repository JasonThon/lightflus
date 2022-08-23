package main

import (
	"context"
	"flag"
	log "github.com/sirupsen/logrus"
	"google.golang.org/grpc"
	"strconv"
	"tableflow/alpha/common/probe"
	"time"
)

var (
	target      = flag.String("TARGET", "", "healthcheck target")
	serviceEnum = flag.String("SERVICE", "", "healthcheck service")
	probeEnum   = flag.String("PROBE", "", "healthcheck probe")
	methodName  = flag.String("METHOD", "", "healthcheck method name")
)

func main() {
	service, err := strconv.Atoi(*serviceEnum)
	if err != nil {
		log.Fatal(err)
	}

	nodeType := probe.ProbeRequest_NodeType(service)

	probeType, err := strconv.Atoi(*probeEnum)
	if err != nil {
		log.Fatal(err)
	}

	ctx, cancelFunc := context.WithTimeout(context.Background(), time.Second*5)
	defer cancelFunc()

	conn, err := grpc.Dial(*target)
	if err != nil {
		log.Fatal(err)
	}
	var resp probe.ProbeResponse
	err = conn.Invoke(ctx, *methodName, &probe.ProbeRequest{
		NodeType:  nodeType,
		ProbeType: probe.ProbeRequest_ProbeType(probeType),
	}, &resp)

	log.Fatalf("health check failed: %v", err)
}
