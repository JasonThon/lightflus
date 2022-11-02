package client

import (
	"context"
	"errors"
	"fmt"
	"github.com/go-resty/resty/v2"
	proto2 "github.com/golang/protobuf/proto"
	"lightflus/common"
	"lightflus/proto/apiserver"
)

type ApiServiceClient struct {
	ctx context.Context
}

func NewApiServiceClient(ctx context.Context) *ApiServiceClient {
	return &ApiServiceClient{ctx: ctx}
}

func (c *ApiServiceClient) CreateResource(req *apiserver.CreateResourceRequest) (*apiserver.CreateResourceResponse, error) {
	var resp apiserver.CreateResourceResponse
	url := common.LightflusUrl + common.CreateResourcePath
	if err := c.sendMessage(resty.MethodPost, url, req, &resp); err != nil {
		return nil, err
	}

	return &resp, nil
}

func (c *ApiServiceClient) GetResource(req *apiserver.GetResourceRequest) (*apiserver.GetResourceResponse, error) {
	var resp apiserver.GetResourceResponse
	url := common.LightflusUrl + common.GetResourcePath(req.ResourceId)
	if err := c.sendMessage(resty.MethodGet, url, req, &resp); err != nil {
		return nil, err
	}

	return &resp, nil
}

func (c *ApiServiceClient) sendMessage(method, url string, req, resp proto2.Message) error {
	client := resty.New()
	request := client.NewRequest()
	request.SetContext(c.ctx)
	body, err := proto2.Marshal(req)
	if err != nil {
		return err
	}
	request.SetBody(body)
	request.SetHeader("Content-Type", common.ApplicationStream)
	response, err := request.Execute(method, url)
	if !response.IsSuccess() {
		return errors.New(fmt.Sprintf("statusCode: %d, errMsg: %s", response.StatusCode(), response.String()))
	}
	if err = proto2.Unmarshal(response.Body(), resp); err != nil {
		return err
	}

	return nil
}

func (c *ApiServiceClient) ListResources(req *apiserver.ListResourcesRequest) (*apiserver.ListResourcesResponse, error) {
	var resp apiserver.ListResourcesResponse
	url := common.LightflusUrl + common.ListResourcesPath(req.Namespace, req.ResourceType)
	if err := c.sendMessage(resty.MethodGet, url, req, &resp); err != nil {
		return nil, err
	}

	return &resp, nil
}
