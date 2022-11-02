/*
Copyright © 2022 NAME HERE <EMAIL ADDRESS>

*/
package cmd

import (
	"fmt"
	"lightflus/client"
	"lightflus/common"
	"lightflus/common/formatter"
	"lightflus/proto/apiserver"
	common2 "lightflus/proto/common"
	"log"
	"os"
	"strings"

	"github.com/spf13/cobra"
)

const taskFunctionWithPlaceHolder = "function $taskName(ctx: ExecutionContext){ \n " +
	"//implement task \n " +
	"} \n " +
	"let ctx = ExecutionContext.new(); \n " +
	"$taskName(ctx);"

// taskCmd represents the task command
var taskCmd = &cobra.Command{
	Use:   "lightflus create task [task name]",
	Short: "create a new dataflow task file",
	Long: `A longer description that spans multiple lines and likely contains examples
and usage of using your command. For example:

Cobra is a CLI library for Go that empowers applications.
This application is a tool to generate the needed files
to quickly create a Cobra application.`,
	Run: func(cmd *cobra.Command, args []string) {
		if len(args) == 0 {
			if err := cmd.Help(); err != nil {
				cmd.PrintErr(err)
			}
		}
		resourceType := args[0]

		switch cmd.Parent().GroupID {
		case common.CreateResourceCmdGroupId:
			executeCreateCmd(resourceType, args)
		case common.GetResourceCmdGroupId:
			executeGetResourceCmd(cmd, resourceType, args)
		}
	},
}

func executeGetResourceCmd(cmd *cobra.Command, resourceType string, args []string) {
	ctx := cmd.Context()
	cmdArgs := ctx.Value(common.ResourceCmdArgKey).(*common.ResourceCmdArgs)
	format := formatter.NewResourceInfoFormatter()
	switch resourceType {
	case common.DataflowResourceType:
		if len(args) <= 1 {
			cmd.PrintErrf("taskId is required")
		}

		taskId := args[1]
		cli := client.NewApiServiceClient(ctx)
		response, err := cli.GetResource(&apiserver.GetResourceRequest{
			ResourceId: &common2.ResourceId{
				ResourceId:  taskId,
				NamespaceId: cmdArgs.Namespace,
			},
		})
		if err != nil {
			cmd.PrintErr(err)
		}
		resource := response.GetResource()
		format.AppendResource(resource)
	case common.DataflowSetResourceType:
		cli := client.NewApiServiceClient(ctx)
		response, err := cli.ListResources(&apiserver.ListResourcesRequest{
			Namespace:    cmdArgs.Namespace,
			ResourceType: apiserver.ResourceTypeEnum_RESOURCE_TYPE_ENUM_DATAFLOW,
		})

		if err != nil {
			cmd.PrintErr(err)
		}
		resources := response.GetResources()
		for _, resource := range resources {
			format.AppendResource(resource)
		}
	}

	format.Println()
}

func executeCreateCmd(resourceType string, args []string) {
	switch resourceType {
	case common.TaskResourceType:
		executeCreateTaskCmd(args)
	}
}

func executeCreateTaskCmd(args []string) {
	if len(args) == 0 {
		log.Fatalf("task name is required")
	}
	taskName := args[0]
	taskFile := strings.Replace(taskFunctionWithPlaceHolder, "$taskName", taskName, -1)

	file, err := os.Create(fmt.Sprintf("%s.ts", taskName))
	if err != nil {
		log.Fatal(err)
	}
	_, err = file.WriteString(taskFile)
	if err != nil {
		log.Fatal(err)
	}
	if err = file.Close(); err != nil {
		log.Fatal(err)
	}
}

func init() {
	createCmd.AddCommand(taskCmd)
	getCmd.AddCommand(taskCmd)

	// Here you will define your flags and configuration settings.

	// Cobra supports Persistent Flags which will work for this command
	// and all subcommands, e.g.:
	// taskCmd.PersistentFlags().String("foo", "", "A help for foo")

	// Cobra supports local flags which will only run when this command
	// is called directly, e.g.:
	// taskCmd.Flags().BoolP("toggle", "t", false, "Help message for toggle")
}
