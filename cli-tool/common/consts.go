package common

import (
	"lightflus/proto/apiserver"
	"lightflus/proto/common"
	"os"
	"strconv"
	"strings"
)

const (
	CreateResourceCmdGroupId = "createResource"
	GetResourceCmdGroupId    = "getResource"
)

const (
	ServiceEndpointEnv = "LIGHTFLUS_SERVICE"
	ServiceToken       = "LIGHTFLUS_TOKEN"
	ApplicationStream  = "application/octet-stream"
)

var (
	LightflusUrl            = os.Getenv(ServiceEndpointEnv)
	CreateResourcePath      = "/resources/create"
	GetResourcePathPrefix   = "/resources/get"
	ListResourcesPathPrefix = "/resources/list"
)

func GetResourcePath(resourceId *common.ResourceId) string {
	return strings.Join([]string{GetResourcePathPrefix, resourceId.GetNamespaceId(), resourceId.GetResourceId()}, "/")
}

func ListResourcesPath(namespace string, resourceType apiserver.ResourceTypeEnum) string {
	return strings.Join([]string{ListResourcesPathPrefix, namespace, strconv.Itoa(int(resourceType))}, "/")
}
