# timeturner

Command line utility for manipulating date-time strings

## Installation

To install with **Homebrew**:

`brew install jrduncans/timeturner/timeturner`

To install with **cargo**:

`cargo install timeturner`

For use in **Alfred** download the [latest release](https://github.com/jrduncans/timeturner/releases/download/v1.2.0/timeturner.alfredworkflow-1.2.0.alfredworkflow)

## Usage

`timeturner 1575149020890`

> 2019-11-30T21:23:40.890Z
>
> 2019-11-30T13:23:40.890-08:00

`timeturner 2019-11-30T13:27:45-08:00`

> 2019-11-30T21:27:45.000Z
>
> 1575149265000

`timeturner '03 Feb 2020 01:03:10.534'`

> 2020-02-03T01:03:10.534Z
>
> 2020-02-02T17:03:10.534-08:00
>
> 1580691790534
