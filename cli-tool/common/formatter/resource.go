package formatter

import (
	"github.com/jedib0t/go-pretty/v6/table"
	"lightflus/proto/apiserver"
	"strings"
)

var header = table.Row{"RESOURCE_ID", "RESOURCE_NAME", "RESOURCE_TYPE", "STATUS", "LIVE TIME"}

type ResourceInfoFormatter struct {
	writer table.Writer
}

func (f *ResourceInfoFormatter) AppendResource(resource *apiserver.Resource) {
	resourceTypeName := strings.TrimPrefix(apiserver.ResourceTypeEnum_name[int32(resource.ResourceType)], "RESOURCE_TYPE_ENUM_")
	resourceStatusName := strings.TrimPrefix(apiserver.ResourceStatusEnum_name[int32(resource.Status)], "RESOURCE_STATUS_ENUM_")
	row := table.Row{resource.ResourceId, resource.ResourceName, resourceTypeName, resourceStatusName}
	f.writer.AppendRow(row)
}

func (f *ResourceInfoFormatter) Println() {
	println(f.writer.Render())
}

func NewResourceInfoFormatter() *ResourceInfoFormatter {
	writer := table.NewWriter()
	writer.AppendHeader(header)
	return &ResourceInfoFormatter{
		writer: writer,
	}
}
