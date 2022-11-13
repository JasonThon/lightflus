package formatter

import (
	"lightflus/proto/apiserver"
	"lightflus/proto/common"
	"testing"
)

func TestResourceFormatterPrintln(t *testing.T) {
	formatter := NewResourceInfoFormatter()
	formatter.AppendResource(&apiserver.Resource{
		ResourceId: &common.ResourceId{
			ResourceId: "resourceId",
		},
	})

	formatter.Println()
}
