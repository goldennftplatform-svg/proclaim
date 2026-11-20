<!-- SPDX-License-Identifier: MIT AND Apache-2.0 -->

# LP-0023 adoption tracker

**Snapshot:** 2026-11-19. **Status:** early draft; localnet only. Testnet 0.4
deployment is pending, and no adoption criterion is met. Values below are the
current honest counts — nothing here is projected or inflated.

Repo: <https://github.com/goldennftplatform-svg/proclaim> (MIT AND Apache-2.0)

## Criteria and current values

| # | LP-0023 adoption criterion | Target | Current value (honest) | Status | Evidence link | Owner | Next action |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | Independent third-party Logos UI modules integrating the SDK | 5 | **0/5** — none yet; the `ui/registry/` module is a first-party scaffold only | Not started | [UI scaffold](https://github.com/goldennftplatform-svg/proclaim/tree/master/ui/registry) · [Integration guide](https://github.com/goldennftplatform-svg/proclaim/blob/master/docs/INTEGRATION.md) | Unassigned — recruit external authors | Invite independent module authors via #builder-hub; record each module's repo and the entry it registers |
| 2a | Registered programs | 50 | **1/50** — registry itself ("LP-0023 Registry v5"), **localnet only** | In progress | [LP0023.md evidence log](https://github.com/goldennftplatform-svg/proclaim/blob/master/LP0023.md) · register tx `38bba96b…` | Project maintainer | Deploy to testnet 0.4; help external developers register their own programs |
| 2b | Distinct independent author accounts | 20 | **1/20** — one author, localnet | Not started | [LP0023.md evidence log](https://github.com/goldennftplatform-svg/proclaim/blob/master/LP0023.md) | Unassigned — community | Recruit independent authors; count one author per distinct account (see below) |
| 3 | Entries with an independent verified source claim | 20 | **1/20** — localnet, from a verifier account distinct from the author | In progress | [verify-source.sh](https://github.com/goldennftplatform-svg/proclaim/blob/master/scripts/verify-source.sh) · verify tx `4d31a6cf…` | Project maintainer + independent verifiers | Ask other developers to run the clean-room rebuild and submit `mark-verified` from their own accounts |
| 4 | Sustained activity: ≥10 registrations/month for ≥2 months | ≥10/mo × 2 mo | **0 months** — not started | Not started | [ROADMAP Phase D](https://github.com/goldennftplatform-svg/proclaim/blob/master/ROADMAP.md) | Unassigned — community | Begin after testnet 0.4; record monthly registration counts with timestamps |
| 5 | Discord testimonials | 20 | **0/20** — none | Not started | n/a (no links yet) | Unassigned | Collect opt-in, timestamped public links from independent users; never write these yourself |
| 6 | X/Twitter testimonials | 20 | **0/20** — none | Not started | n/a (no links yet) | Unassigned | Collect opt-in, timestamped public links from independent accounts |
| 7 | macOS coverage in adoption evidence | required | **None** — Linux verified only; macOS not covered | Not started | [ROADMAP Phase D](https://github.com/goldennftplatform-svg/proclaim/blob/master/ROADMAP.md) | Unassigned | Verify `verify-source.sh` and the FFI/SDK build on macOS with an independent macOS user |

## How to count honestly

These rules apply to every row above and to any update of this table.

- **Distinct author accounts:** count one author per independent account/identity,
  not per entry. Repeated registrations from the same account do not increase
  the distinct-author count (row 2b), and the same account cannot vouch for
  itself as an independent verifier or testimonial.
- **No manufactured volume:** do not create fake registrations, alternate
  accounts to simulate multiple authors, self-verifications, seeded testimonials,
  or paid/ incentivized posts presented as organic. Registrations must come from
  developers actually deploying their own programs. Note that
  `deployer_claimed` is a self-assertion and is not evidence of independent use.
- **Timestamped public evidence:** every counted item needs a durable public link
  (repo/commit, transaction hash, module repo, or the public message/post) and
  the date it appeared. Add the link and date to the Evidence column at the same
  time the count increases. Localnet records are labelled as localnet and are not
  presented as official-zone/testnet adoption.
- **No self-testimonials from the same account:** a testimonial for rows 5 and 6
  must be written and posted by a user other than the submitter, from their own
  account, with a link. Do not write, rewrite, or cross-post testimonials from
  the project's own accounts, and do not count the same person's Discord and X
  posts as two independent supporters without noting they are from one person.

## Update checklist

Before changing any number in this tracker:

1. Confirm the claimed criterion is actually met (not just planned or localnet-only).
2. Attach or update the timestamped public evidence link in the Evidence column.
3. Confirm the item comes from an independent account and is not manufactured volume.
4. Keep the snapshot date and status line at the top current.
