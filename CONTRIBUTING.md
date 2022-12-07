# Introduction

Hi, we're very happy that you want to contribute to Lightflus. We welcome anyone to make contribution to Lightflus, regardless small or big. 

This is the guideline for you to make you feel much easier on your trip. Please make sure you have read it carefully before contributing. 

# Issue
At the early stage of Lightflus, we're very closely watching the issue of Lightflus. If you want to commit your issue to us, you can directly submit it on web page, however, **please follow the template we provide for you. It's important.**

# Pull Request
We also provide a pull request template for you. Before you push your code, please confirm:

1. Your code is meaningful. It can be document modification, bug fix, new features, new test cases, etc.
2. Commit message must be clear enough, like: 
* `[feat/<jira-number>] <commit message>`
* `[bugfix: brief description] <commit message about the bug you fix>`
* `[tests: brief description] <commit message about your test cases>`
* `[document: brief description] <commit message about the document info>`
3. **Each pull request must be reviewed**. **Please assign to me** (if not, maybe I will not handle this).
4. CI will be triggered after pull request is launched. **I will review your code after CI is success. If CI is failed, please fix it first**

**If you have any other question about PR, please let me know~**

# Development Guideline

1. Dev Tool: I use **vscode** and **WebStorm** as my dev tool. It's not strict, you can use any tool if you like.
2. Start Lightflus: please follow [README.md](README.md) to start lightflus locally. **We recommend you to try to start lightflus in you IDE because you can debug your code much easier**;
3. **Unit tests is mandatory for new features and bug fix**. If your pull request has no unit test or unit test does not cover enough cases, we will not approve it.
4. **New Sink and source implementation must have integration test, so does the old one if code changes**. If your pull request's impact scope contains source or sink but has no integration test, we will not approve it. All integration tests must locate in the path `${WORKSPACE_ROOT}/tests`. If you want to start up a new external system (for example, `elasticsearch` should be up for integration test), you can add it in `${PROJECT_ROOT}/.github/workflow.yml` file.
5. In some cases, **test manually is necessary**. **You must follow the template** of pull request to tell us how to test your pull request. **If your code is covered by CI or no need for test, please explain it**. 
6. `cargo check` is very useful when you're developing. Run it frequently will save much time for you. 

**If you have any question about developing Lightflus, please let me know~**

