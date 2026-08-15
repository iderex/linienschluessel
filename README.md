# linienschluessel

A measured spectrum holds hundreds to thousands of lines, and assigning them
to level transitions is handwork. It goes wrong: the J quantum numbers in Tm
II had to be revised after they were assigned wrongly. The task itself is
formally clean. Given observed wavelengths with intensities and predicted
levels, find the assignment that satisfies the Ritz combination principle,
respects the selection rules and best explains the intensity ratios. The
answer comes back with uncertainties, not as one line of output. NIST supplies
levels and lines separately, so the method is validated against already-solved
spectra before it is turned loose on the unidentified lines in solar, fusion
and laboratory spectra.

Planning happens on the issue tracker first. Every decision that shapes
the architecture is written down there with its reasons before the code
that depends on it exists.

See [NOTICE.md](NOTICE.md) for the intended-use notice.

## Your data, and what you are responsible for

Four things to know before a spectrum you care about goes near this. Each one
names the file that governs it or the command that shows it, so none of them
rests on this page alone.

The data you load stays on this machine. A run reads files and writes files and
does nothing else, which is the rule in
[docs/data-on-the-host.md](docs/data-on-the-host.md), stated there with its
reasons. The command that reads it is the word list that file carries:

    git grep -nE 'std::net|TcpStream|TcpListener|UdpSocket|reqwest|hyper|ureq|curl::|tokio::net|SocketAddr|to_socket_addrs' -- 'crates' ; echo "grep exit=$?"
    grep exit=1

A clean result there is worth more now than it once was, and it is still not
the property itself; the same file says why. There are thirteen crates here
and eleven of them hold no code, so the search covers two, and those two are
the readers that take a file from your disk. What will be worth something is a
run of the whole pipeline with no network route available, and that does not
exist.

An unpublished spectrum loaded here stays unpublished, under the same rule and
the same file. Nothing transmits, uploads or reports anything: no telemetry, no
usage counting, no crash reporting, no update check, no remote configuration.
Not switched off by default, absent. The check that needs no trust in any of
this is to disconnect the machine and run the tool, which asks for no
administrative rights, and which asks for a tool that is not built yet.

You are responsible for the terms of the data you load. This board cannot know
what you retrieved or under what agreement, so the terms of every source it can
read are collected in [docs/sources.md](docs/sources.md), each entry with the
date those terms were read and the command that read them. The widely held
belief that data from a United States federal agency is free of copyright is
wrong for the one source there, which is why that file exists rather than a
formality.

An answer file carries the attribution your sources require, and it is yours to
keep intact when you pass it on. Which attribution that is comes from
[docs/sources.md](docs/sources.md) rather than from anything written into a
program. No answer file exists yet and nothing derives an attribution from that
file today, which its own last section states and which issue #60 stays open
for.

## How this sits with the notice

[NOTICE.md](NOTICE.md) is the intended-use notice and it is the general
statement: the software is developed for lawful use, and use has to comply with
the laws that apply to you, copyright and data protection included. The four
paragraphs above are not a second copy of that in other words. They are the
specific obligations this board can name, each with a file or a command behind
it, and where the two touch, the notice is the general form and
[docs/sources.md](docs/sources.md) is the concrete one.

The notice says the licence carries the full warranty and liability disclaimer.
That licence exists now and the section below names it, so the two resolve by
being read together. Until 2026-08-08 this place said the opposite, because
there was no licence file to point at.

## License

AGPL-3.0, decided by the maintainer on 2026-08-08. It answers entry 1 of
issue #1 and no other entry in that issue.

The full text is in [LICENSE](LICENSE). Read that file rather than this line,
and if you want the platform's own reading of it, run:

    gh api repos/iderex/linienschluessel --jq '.license.spdx_id'
