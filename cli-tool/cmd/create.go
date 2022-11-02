/*
Copyright © 2022 NAME HERE <EMAIL ADDRESS>

*/
package cmd

import (
	"context"
	"flag"
	"github.com/spf13/cobra"
	"github.com/spf13/pflag"
	"lightflus/common"
	"log"
)

// createCmd represents the create command
var createCmd = &cobra.Command{
	GroupID: common.CreateResourceCmdGroupId,
	Use:     "create",
	Short:   "create resources like task file, cluster",
	Run: func(cmd *cobra.Command, args []string) {
		for _, command := range cmd.Commands() {
			nameFlag := cmd.Flag("name")
			namespaceFlag := cmd.Flag("namespace")
			if nameFlag == nil {
				cmd.PrintErrf("resource name is required")
				return
			}
			ctx := context.WithValue(command.Context(), common.ResourceCmdArgKey, &common.ResourceCmdArgs{
				Name:      nameFlag.Value.String(),
				Namespace: namespaceFlag.Value.String(),
			})

			if err := command.ExecuteContext(ctx); err != nil {
				log.Fatal(err)
			}
		}
	},
}

func init() {
	rootCmd.AddCommand(createCmd)
	createCmd.Flags().AddFlag(pflag.PFlagFromGoFlag(&flag.Flag{
		Name:     "name",
		Usage:    "the name of resource",
		DefValue: "default",
	}))
	createCmd.Flags().AddFlag(pflag.PFlagFromGoFlag(&flag.Flag{
		Name:     "namespace",
		Usage:    "the namespace of resource to create",
		DefValue: "default",
	}))

	// Here you will define your flags and configuration settings.

	// Cobra supports Persistent Flags which will work for this command
	// and all subcommands, e.g.:
	// createCmd.PersistentFlags().String("foo", "", "A help for foo")

	// Cobra supports local flags which will only run when this command
	// is called directly, e.g.:
	// createCmd.Flags().BoolP("toggle", "t", false, "Help message for toggle")
}
