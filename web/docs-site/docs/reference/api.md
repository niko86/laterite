# Python API

Generated from the installed `laterite` wheel. New here? the [Learn path](../learn/index.md)
walks the same surface in order, and the [Cookbook](../cookbook/index.md) is the task-indexed
recipe set. Cross-references below are clickable.

## Reading & validating

::: laterite.read

::: laterite.validate

::: laterite.Ags4File

::: laterite.Report

## Producing & repairing

::: laterite.build_ags4

::: laterite.TranStamp

::: laterite.BuildResult

::: laterite.BuildSaved

::: laterite.fix

::: laterite.FixResult

::: laterite.diff

::: laterite.merge

::: laterite.MergeResult

## Rules

::: laterite.list_rules

::: laterite.fixable_rules

::: laterite.FixableRule

## Querying

::: laterite.AgsQuery

## Excel

::: laterite.from_excel

## Errors

::: laterite.Ags4Error

::: laterite.BadDictError

::: laterite.MergeConflictError

::: laterite.NotAgs4Error

::: laterite.StaleCertError

::: laterite.UnsupportedEditionError

::: laterite.WorldCheckRequiresSourceError

## Type aliases

The enumerated string choices accepted by the API, as `Literal` types so that
editors autocomplete the valid values and type-checkers reject typos. Each is gated in the
test suite against its source of truth.

::: laterite.Edition

::: laterite.Backend

::: laterite.XnMode

::: laterite.BuildMode
