extern crate irc;
extern crate serde;
extern crate serde_json;

use azure_functions::{
    bindings::{HttpRequest, HttpResponse},
    func,
};
use irc::client::prelude::*;
use log::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use std::default::Default;
use std::thread;

fn dispatch_to_irc(event: PushEvent) {
    thread::spawn(move || {
        /*
        * Currently uncertain on how to import a file
        */
        //let config = Config::load("Irc.toml").expect("failed to load Irc.toml");
        let config = Config {
            nickname: Some("otto-gh-bot".to_owned()),
            server: Some("chat.freenode.net".to_owned()),
            channels: Some(vec!["#otto".to_owned()]),
            ..Default::default()
        };

        /*
        * Grabbing a clone of our configured channels before the config
        * moves into IrcClient
        */
        let channels = &config.channels.clone();

        let client = IrcClient::from_config(config)
            .expect("Failed to construct the IrcClient");
        client.identify().unwrap();

        client.for_each_incoming(|irc_msg| {
            let m = irc_msg.to_string();

            /*
            * This effectively means the channel has been synchronized and we can
            * send the message we need to send
            */
            if m.contains("End of /NAMES list") {
                if let Some(channel) = channels {
                    for chan in channel.iter() {
                        info!("Sending message to {}", chan);
                        client.send_privmsg(chan,
                                            format!("{} pushed {} commits to {} in {}  <{}>",
                                                    &event.sender.login,
                                                    &event.commits.len(),
                                                    &event.refname,
                                                    &event.repository.full_name,
                                                    &event.compare))
                            .expect(format!("Failed to send message to channel: {}", chan).as_str());
                        client.send_quit("rusty , over and out")
                            .expect("Failed to send QUIT");
                    }
                }
            }
        }).unwrap();
    });
}

#[func]
pub fn webhook(req: HttpRequest) -> HttpResponse {
    if let Ok(event) = req.body().as_json::<PushEvent>() {
        info!("Parsed push event from {}", event.sender.login);
        dispatch_to_irc(event);
        return "Messages dispatched to IRC".into();
    }
    else {
        return "Failed to parse event".into();
    }
}

#[derive(Serialize, Deserialize)]
struct Sender {
    login: String,
}
#[derive(Serialize, Deserialize)]
struct Repo {
    name: String,
    full_name: String,
}

#[derive(Serialize, Deserialize)]
struct PushEvent {
    #[serde(rename = "ref")]
    refname: String,
    sender: Sender,
    compare: String,
    commits: Vec<Value>,
    repository: Repo,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_push_event() {
        /*
         * Example event taken from:
         *   <https://developer.github.com/v3/activity/events/types/#pushevent>
         */
        let push = r#"{
  "ref": "refs/tags/simple-tag",
  "before": "6113728f27ae82c7b1a177c8d03f9e96e0adf246",
  "after": "0000000000000000000000000000000000000000",
  "created": false,
  "deleted": true,
  "forced": false,
  "base_ref": null,
  "compare": "https://github.com/Codertocat/Hello-World/compare/6113728f27ae...000000000000",
  "commits": [

  ],
  "head_commit": null,
  "repository": {
    "id": 186853002,
    "node_id": "MDEwOlJlcG9zaXRvcnkxODY4NTMwMDI=",
    "name": "Hello-World",
    "full_name": "Codertocat/Hello-World",
    "private": false,
    "owner": {
      "name": "Codertocat",
      "email": "21031067+Codertocat@users.noreply.github.com",
      "login": "Codertocat",
      "id": 21031067,
      "node_id": "MDQ6VXNlcjIxMDMxMDY3",
      "avatar_url": "https://avatars1.githubusercontent.com/u/21031067?v=4",
      "gravatar_id": "",
      "url": "https://api.github.com/users/Codertocat",
      "html_url": "https://github.com/Codertocat",
      "followers_url": "https://api.github.com/users/Codertocat/followers",
      "following_url": "https://api.github.com/users/Codertocat/following{/other_user}",
      "gists_url": "https://api.github.com/users/Codertocat/gists{/gist_id}",
      "starred_url": "https://api.github.com/users/Codertocat/starred{/owner}{/repo}",
      "subscriptions_url": "https://api.github.com/users/Codertocat/subscriptions",
      "organizations_url": "https://api.github.com/users/Codertocat/orgs",
      "repos_url": "https://api.github.com/users/Codertocat/repos",
      "events_url": "https://api.github.com/users/Codertocat/events{/privacy}",
      "received_events_url": "https://api.github.com/users/Codertocat/received_events",
      "type": "User",
      "site_admin": false
    },
    "html_url": "https://github.com/Codertocat/Hello-World",
    "description": null,
    "fork": false,
    "url": "https://github.com/Codertocat/Hello-World",
    "forks_url": "https://api.github.com/repos/Codertocat/Hello-World/forks",
    "keys_url": "https://api.github.com/repos/Codertocat/Hello-World/keys{/key_id}",
    "collaborators_url": "https://api.github.com/repos/Codertocat/Hello-World/collaborators{/collaborator}",
    "teams_url": "https://api.github.com/repos/Codertocat/Hello-World/teams",
    "hooks_url": "https://api.github.com/repos/Codertocat/Hello-World/hooks",
    "issue_events_url": "https://api.github.com/repos/Codertocat/Hello-World/issues/events{/number}",
    "events_url": "https://api.github.com/repos/Codertocat/Hello-World/events",
    "assignees_url": "https://api.github.com/repos/Codertocat/Hello-World/assignees{/user}",
    "branches_url": "https://api.github.com/repos/Codertocat/Hello-World/branches{/branch}",
    "tags_url": "https://api.github.com/repos/Codertocat/Hello-World/tags",
    "blobs_url": "https://api.github.com/repos/Codertocat/Hello-World/git/blobs{/sha}",
    "git_tags_url": "https://api.github.com/repos/Codertocat/Hello-World/git/tags{/sha}",
    "git_refs_url": "https://api.github.com/repos/Codertocat/Hello-World/git/refs{/sha}",
    "trees_url": "https://api.github.com/repos/Codertocat/Hello-World/git/trees{/sha}",
    "statuses_url": "https://api.github.com/repos/Codertocat/Hello-World/statuses/{sha}",
    "languages_url": "https://api.github.com/repos/Codertocat/Hello-World/languages",
    "stargazers_url": "https://api.github.com/repos/Codertocat/Hello-World/stargazers",
    "contributors_url": "https://api.github.com/repos/Codertocat/Hello-World/contributors",
    "subscribers_url": "https://api.github.com/repos/Codertocat/Hello-World/subscribers",
    "subscription_url": "https://api.github.com/repos/Codertocat/Hello-World/subscription",
    "commits_url": "https://api.github.com/repos/Codertocat/Hello-World/commits{/sha}",
    "git_commits_url": "https://api.github.com/repos/Codertocat/Hello-World/git/commits{/sha}",
    "comments_url": "https://api.github.com/repos/Codertocat/Hello-World/comments{/number}",
    "issue_comment_url": "https://api.github.com/repos/Codertocat/Hello-World/issues/comments{/number}",
    "contents_url": "https://api.github.com/repos/Codertocat/Hello-World/contents/{+path}",
    "compare_url": "https://api.github.com/repos/Codertocat/Hello-World/compare/{base}...{head}",
    "merges_url": "https://api.github.com/repos/Codertocat/Hello-World/merges",
    "archive_url": "https://api.github.com/repos/Codertocat/Hello-World/{archive_format}{/ref}",
    "downloads_url": "https://api.github.com/repos/Codertocat/Hello-World/downloads",
    "issues_url": "https://api.github.com/repos/Codertocat/Hello-World/issues{/number}",
    "pulls_url": "https://api.github.com/repos/Codertocat/Hello-World/pulls{/number}",
    "milestones_url": "https://api.github.com/repos/Codertocat/Hello-World/milestones{/number}",
    "notifications_url": "https://api.github.com/repos/Codertocat/Hello-World/notifications{?since,all,participating}",
    "labels_url": "https://api.github.com/repos/Codertocat/Hello-World/labels{/name}",
    "releases_url": "https://api.github.com/repos/Codertocat/Hello-World/releases{/id}",
    "deployments_url": "https://api.github.com/repos/Codertocat/Hello-World/deployments",
    "created_at": 1557933565,
    "updated_at": "2019-05-15T15:20:41Z",
    "pushed_at": 1557933657,
    "git_url": "git://github.com/Codertocat/Hello-World.git",
    "ssh_url": "git@github.com:Codertocat/Hello-World.git",
    "clone_url": "https://github.com/Codertocat/Hello-World.git",
    "svn_url": "https://github.com/Codertocat/Hello-World",
    "homepage": null,
    "size": 0,
    "stargazers_count": 0,
    "watchers_count": 0,
    "language": "Ruby",
    "has_issues": true,
    "has_projects": true,
    "has_downloads": true,
    "has_wiki": true,
    "has_pages": true,
    "forks_count": 1,
    "mirror_url": null,
    "archived": false,
    "disabled": false,
    "open_issues_count": 2,
    "license": null,
    "forks": 1,
    "open_issues": 2,
    "watchers": 0,
    "default_branch": "master",
    "stargazers": 0,
    "master_branch": "master"
  },
  "pusher": {
    "name": "Codertocat",
    "email": "21031067+Codertocat@users.noreply.github.com"
  },
  "sender": {
    "login": "Codertocat",
    "id": 21031067,
    "node_id": "MDQ6VXNlcjIxMDMxMDY3",
    "avatar_url": "https://avatars1.githubusercontent.com/u/21031067?v=4",
    "gravatar_id": "",
    "url": "https://api.github.com/users/Codertocat",
    "html_url": "https://github.com/Codertocat",
    "followers_url": "https://api.github.com/users/Codertocat/followers",
    "following_url": "https://api.github.com/users/Codertocat/following{/other_user}",
    "gists_url": "https://api.github.com/users/Codertocat/gists{/gist_id}",
    "starred_url": "https://api.github.com/users/Codertocat/starred{/owner}{/repo}",
    "subscriptions_url": "https://api.github.com/users/Codertocat/subscriptions",
    "organizations_url": "https://api.github.com/users/Codertocat/orgs",
    "repos_url": "https://api.github.com/users/Codertocat/repos",
    "events_url": "https://api.github.com/users/Codertocat/events{/privacy}",
    "received_events_url": "https://api.github.com/users/Codertocat/received_events",
    "type": "User",
    "site_admin": false
  }
}
        "#;

        let event = serde_json::from_str::<PushEvent>(push).expect("Failed to deserialized event");
        assert_eq!("Codertocat", event.sender.login);
        assert_eq!("refs/tags/simple-tag", event.refname);
    }
}
