package common

type ResourceType int

const (
	Unspecified ResourceType = iota
	Cluster     ResourceType = 1

	ResourceCmdArgKey = "resource_cmd_arg"
)

const (
	TaskResourceType        = "task"
	DataflowResourceType    = "dataflow"
	DataflowSetResourceType = "dataflows"
)

type ResourceCmdArgs struct {
	Name      string
	Namespace string
}
