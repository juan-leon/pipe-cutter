# pipe-cutter
Command line tool to filter out some of the data piped in

## What does pipe-cutter?

pipe-cutter is a tool to use as pipe to limit what is read from stdin (in time
or size).

Example: `pipe-cutter --tail nginx.log --seconds 10 > 10-secs.log`

Example: `tail -f nginx.log gnunicorn.log | pipe-cutter --bytes 3000`

```
USAGE:
    pipe-cutter [OPTIONS]

FLAGS:
    -h, --help
            Prints help information

    -V, --version
            Prints version information


OPTIONS:
        --bytes <bytes>
            Stop after reading that many bytes.  If reading from stdin, try to honor newlines

        --seconds <seconds>
            Stop reading after that many seconds

        --tail <tail>
            Read changes in this file ("tail -f" style), as opposed to using stdin
```

I sometimes use it along [lowcharts](https://github.com/juan-leon/lowcharts) to
visualize chunks of information coming from a logfile/strace/tcpdump/etc.

## Is it really needed?

You can get pipe-cutter functionality by using standard unix tools.  For
example, `pipe-cutter --tail FILE --seconds 60 --bytes 10000` could be mimicked
by doing `timeout 60 tail -f FILE | head -c 10000`.

### Installation

#### Via release

Go over https://github.com/juan-leon/pipe-cutter/releases/ and download the binary
you want.  Decompress the file and copy the binary to your path.

#### Via local compilation

```
$ git clone https://github.com/juan-leon/pipe-cutter
$ cd pipe-cutter
$ cargo install --path .
```

### Contributing

Feedback, ideas and pull requests are welcomed.
