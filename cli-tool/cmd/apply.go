/*
Copyright © 2022 NAME HERE <EMAIL ADDRESS>

*/
package cmd

import (
	"flag"
	"fmt"
	"github.com/spf13/pflag"
	"os/exec"
	"strings"

	"github.com/spf13/cobra"
)

// applyCmd represents the apply command
var applyCmd = &cobra.Command{
	Use:   "apply",
	Short: "A brief description of your command",
	Long: `A longer description that spans multiple lines and likely contains examples
and usage of using your command. For example:

Cobra is a CLI library for Go that empowers applications.
This application is a tool to generate the needed files
to quickly create a Cobra application.`,
	Run: func(cmd *cobra.Command, args []string) {
		flag := cmd.Flag("f")
		filepath := flag.Value.String()
		if err := exec.Command(fmt.Sprintf("npx tsc -t %s", filepath)); err != nil {
			cmd.PrintErr(err)
			return
		}
		split := strings.Split(filepath, "/")
		name := split[len(split)-1]
		filename := strings.Split(name, ".ts")[0]
		split[len(split)-1] = filename + ".js"
		filepath = strings.Join(split, "/")
		if err := exec.Command(fmt.Sprintf("node %s", filepath)).Wait(); err != nil {
			cmd.PrintErr(err)
			return
		}
		if err := exec.Command(fmt.Sprintf("rm -rf %v", filepath)); err != nil {
			cmd.Printf("clean compiled file failed")
		}
	},
}

func init() {
	rootCmd.AddCommand(applyCmd)
	applyCmd.Flags().AddFlag(pflag.PFlagFromGoFlag(&flag.Flag{
		Name:  "f",
		Usage: "",
	}))

	// Here you will define your flags and configuration settings.

	// Cobra supports Persistent Flags which will work for this command
	// and all subcommands, e.g.:
	// applyCmd.PersistentFlags().String("foo", "", "A help for foo")

	// Cobra supports local flags which will only run when this command
	// is called directly, e.g.:
	// applyCmd.Flags().BoolP("toggle", "t", false, "Help message for toggle")
}
