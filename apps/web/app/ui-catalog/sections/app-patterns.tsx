"use client";

import { useState } from "react";
import {
  IconHash,
  IconMessageCircle,
  IconPlus,
  IconSettings,
  IconSpeakerphone,
  IconVolume,
} from "@tabler/icons-react";

import { ChannelRail } from "@/components/chat/channel-rail";
import { Composer } from "@/components/chat/composer";
import { MessageRow } from "@/components/chat/message-row";
import { MessageTimeline } from "@/components/chat/message-timeline";
import { PresenceDot } from "@/components/chat/presence";
import { Bar as ContentTabsBar } from "@/components/content-tabs/bar";
import { BulkActions } from "@/components/hubs/bulk-actions";
import { ContactDiscoveryResults, type ContactDiscoveryUser } from "@/components/hubs/contact-discovery-results";
import { ContactRequestSection, type ContactRequest } from "@/components/hubs/contact-request-section";
import { ItemActions } from "@/components/hubs/item-actions";
import { Surface as HubSurface, type ItemData } from "@/components/hubs/surface";
import { Toolbar as HubToolbar } from "@/components/hubs/toolbar";
import { Controls as ProfileControls } from "@/components/profile/controls";
import type { Profile } from "@/components/profile/types";
import { Icon as WorkspaceIcon } from "@/components/server-workspace/icon";
import { MemberCard } from "@/components/server-workspace/member-card";
import type { Member } from "@/components/server-workspace/types";
import { VoiceParticipantRow } from "@/components/server-workspace/voice-participant-row";
import { Button } from "@/components/ui/buttons/button";
import { Badge } from "@/components/ui/display/badge";
import { Menu } from "@/components/ui/navigation/menu";
import { Panel } from "@/components/ui/surfaces/panel";
import { Button as SettingsButton } from "@/components/settings/button";
import { Panel as SettingsPanel } from "@/components/settings/panel";
import { Row as SettingsRow } from "@/components/settings/row";
import { Select as SettingsSelect } from "@/components/settings/select";
import { Toggle as SettingsToggle } from "@/components/settings/toggle";
import { Value as SettingsValue } from "@/components/settings/value";
import type { ServerChannelMessage } from "@/lib/api";
import type { HubLayout } from "@/lib/hub-state";
import type { NavLayout } from "@/lib/workspace-preferences";

import type { SectionId } from "../data";
import { Example } from "../example";
import { Section } from "../section";

import styles from "../styles.module.css";

const profile: Profile = {
  active: true,
  initials: "AL",
  name: "Aline Costa",
  status: "Online in Atlas Team",
};

const messageAuthors: Record<string, { handle: string; label: string }> = {
  usr_aline: { handle: "aline", label: "Aline Costa" },
  usr_diogo: { handle: "diogo", label: "Diogo Martins" },
  usr_mara: { handle: "mara", label: "Mara Silva" },
};

const messages: ServerChannelMessage[] = [
  {
    author_id: "usr_mara",
    channel_id: "general",
    channel_seq: 41,
    content: "The sidebar spacing feels consistent again after the menu cleanup.",
    created_at: "2026-06-11T09:42:00.000Z",
    mentions: [],
    message_id: "msg-sidebar-spacing",
  },
  {
    author_id: "usr_aline",
    channel_id: "general",
    channel_seq: 42,
    content: "I added the message catalog examples so we can compare row states in one place.",
    created_at: "2026-06-11T09:46:00.000Z",
    edited_at: "2026-06-11T09:48:00.000Z",
    mentions: ["usr_diogo"],
    message_id: "msg-catalog-examples",
    reply_to_message_id: "msg-sidebar-spacing",
  },
  {
    author_id: "usr_diogo",
    channel_id: "general",
    channel_seq: 43,
    content: "Good. Keep the primitive docs separate from app-specific patterns.",
    created_at: "2026-06-11T09:51:00.000Z",
    mentions: [],
    message_id: "msg-primitive-docs",
  },
];

const hubItems: ItemData[] = [
  { id: "atlas", muted: false, name: "Atlas Team", pinned: true, unread: 12 },
  { id: "relay", muted: true, name: "Relay Lab", pinned: false, unread: 0 },
  { id: "design", muted: false, name: "Design Guild", pinned: false, unread: 4 },
];

const members: Member[] = [
  {
    identityId: "usr_aline",
    joinedAt: "2026-05-18T14:20:00.000Z",
    lastActive: "Active now",
    muted: false,
    pinned: true,
    presence: "online",
    role: "Admin",
    title: "Product systems",
    unread: 3,
  },
  {
    identityId: "usr_mara",
    joinedAt: "2026-05-22T11:10:00.000Z",
    lastActive: "Away for 12m",
    muted: true,
    pinned: false,
    presence: "away",
    role: "Member",
    title: "Client runtime",
    unread: 0,
  },
];

const contactRequests: ContactRequest[] = [
  {
    created_at: "2026-06-10T16:20:00.000Z",
    request_id: "req-mara",
    requester_identity_id: "usr_mara",
    status: "pending",
    target_identity_id: "usr_aline",
  },
];

const discoveredContacts: ContactDiscoveryUser[] = [
  {
    can_send_friend_request: true,
    display_name: "Nuno Reis",
    has_pending_inbound_request: false,
    has_pending_outbound_request: false,
    identity_id: "usr_nuno",
    relationship_state: "none",
    shared_server_count: 2,
  },
  {
    can_send_friend_request: false,
    display_name: "Mara Silva",
    has_pending_inbound_request: true,
    has_pending_outbound_request: false,
    identity_id: "usr_mara",
    relationship_state: "pending_inbound",
    shared_server_count: 1,
  },
];

function authorHandle(identityId: string): string {
  return messageAuthors[identityId]?.handle ?? identityId.slice(0, 8);
}

function authorLabel(identityId: string): string {
  return messageAuthors[identityId]?.label ?? identityId;
}

function formatTimestamp(value?: string): string {
  if (!value) {
    return "unknown";
  }

  return new Intl.DateTimeFormat("en", {
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
    month: "short",
  }).format(new Date(value));
}

function shortIdentity(identityId: string): string {
  return identityId.replace("usr_", "@");
}

export function AppPatternsSections({ isVisible }: { isVisible: (sectionId: SectionId) => boolean }) {
  const [composerValue, setComposerValue] = useState("Draft a quick channel update...");
  const [hubLayout, setHubLayout] = useState<HubLayout>("list");
  const [hubMutedOnly, setHubMutedOnly] = useState(false);
  const [hubPinnedOnly, setHubPinnedOnly] = useState(false);
  const [hubSearch, setHubSearch] = useState("");
  const [hubSelectedIds, setHubSelectedIds] = useState<Set<string>>(() => new Set(["atlas"]));
  const [hubUnreadOnly, setHubUnreadOnly] = useState(false);
  const [microphoneMuted, setMicrophoneMuted] = useState(false);
  const [navLayout, setNavLayout] = useState<NavLayout>("sidebar");
  const [profileCollapsed, setProfileCollapsed] = useState(false);
  const [settingsAlerts, setSettingsAlerts] = useState(true);
  const [settingsDensity, setSettingsDensity] = useState("comfortable");
  const [soundMuted, setSoundMuted] = useState(false);

  function toggleHubSelected(itemId: string): void {
    setHubSelectedIds((current) => {
      const next = new Set(current);

      if (next.has(itemId)) {
        next.delete(itemId);
      } else {
        next.add(itemId);
      }

      return next;
    });
  }

  return (
    <>
      <Section
        description="Message patterns cover the channel rail, timeline rows, composer, mentions, replies, edited state, and presence."
        id="messages"
        title="Messages"
        visible={isVisible("messages")}
      >
        <div className={styles.exampleGrid}>
          <Example title="Channel Rail">
            <div className={styles.catalogChannelRail}>
              <ChannelRail title="Channels">
                <Menu
                  activeId="general"
                  idleBorder={false}
                  items={[
                    {
                      end: (
                        <Badge aria-label="12 unread" shape="counter" size="sm" tone="accent">
                          12
                        </Badge>
                      ),
                      icon: <IconHash aria-hidden="true" />,
                      id: "general",
                      name: "general",
                    },
                    { icon: <IconMessageCircle aria-hidden="true" />, id: "feedback", name: "feedback" },
                    { icon: <IconVolume aria-hidden="true" />, id: "voice", name: "Voice room" },
                  ]}
                  panel={false}
                />
              </ChannelRail>
            </div>
          </Example>

          <Example title="Presence">
            <div className={styles.presenceSamples}>
              <span className={styles.presenceSample}>
                <span className={styles.presenceAvatarFrame}>
                  <span className={styles.presenceAvatar}>AL</span>
                  <PresenceDot aria-label="Online" status="online" />
                </span>
                <span>Online</span>
              </span>
              <span className={styles.presenceSample}>
                <span className={styles.presenceAvatarFrame}>
                  <span className={styles.presenceAvatar}>MS</span>
                  <PresenceDot aria-label="Away" status="away" />
                </span>
                <span>Away</span>
              </span>
            </div>
          </Example>

          <Example title="Timeline And Composer" wide>
            <div className={styles.messageCatalogShell}>
              <MessageTimeline
                bubbleSize="comfortable"
                layout="bubble-cards"
                loadOlderLabel="Load earlier messages"
                onLoadOlder={() => undefined}
              >
                {messages.map((message) => (
                  <MessageRow
                    alignment="conversation-sides"
                    authorHandle={authorHandle}
                    authorLabel={authorLabel}
                    bubbleSize="comfortable"
                    formatTimestamp={formatTimestamp}
                    key={message.message_id}
                    layout="bubble-cards"
                    message={message}
                    onReply={() => undefined}
                    ownMessage={message.author_id === "usr_diogo"}
                    replyTo={message.reply_to_message_id ? messages[0] : undefined}
                    shortIdentity={shortIdentity}
                  />
                ))}
              </MessageTimeline>
              <Composer
                hints={
                  <>
                    <Badge tone="muted">Markdown ready</Badge>
                    <Badge tone="accent">@mentions enabled</Badge>
                  </>
                }
                onChange={setComposerValue}
                onSend={() => undefined}
                placeholder="Message #general"
                replyLabel="Replying to Aline Costa"
                replyText="I added the message catalog examples..."
                sendLabel="Send"
                value={composerValue}
                onCancelReply={() => undefined}
              />
            </div>
          </Example>
        </div>
      </Section>

      <Section
        description="Profile controls document the reusable account card, voice toggles, compact mode, and profile action popup."
        id="profile-controls"
        title="Profile Controls"
        visible={isVisible("profile-controls")}
      >
        <div className={styles.exampleGrid}>
          <Example title="Sidebar Controls" wide>
            <div className={styles.profilePatternFrame}>
              <ProfileControls
                collapsed={profileCollapsed}
                microphoneMuted={microphoneMuted}
                navLayout={navLayout}
                onOpenAudioDevices={() => undefined}
                onSetCollapsed={setProfileCollapsed}
                onSetMicrophoneMuted={setMicrophoneMuted}
                onSetNavLayout={setNavLayout}
                onSetSoundMuted={setSoundMuted}
                placement="sidebar"
                profile={profile}
                soundMuted={soundMuted}
                voiceActionsAvailable
              />
            </div>
          </Example>
        </div>
      </Section>

      <Section
        description="Content tabs are the in-view tab strip used for workspace pages and secondary navigation."
        id="content-tabs"
        title="Content Tabs"
        visible={isVisible("content-tabs")}
      >
        <div className={styles.exampleGrid}>
          <Example title="Scrollable Bar" wide>
            <div className={styles.contentTabCatalogFrame}>
              <ContentTabsBar
                activeId="chat"
                actions={
                  <Button icon={<IconPlus aria-hidden="true" />} size="sm">
                    New
                  </Button>
                }
                canScrollLeft
                canScrollRight
                items={[
                  { icon: IconHash, id: "overview", label: "Overview" },
                  { icon: IconMessageCircle, id: "chat", label: "Chat" },
                  { icon: IconSpeakerphone, id: "voice", label: "Voice" },
                  { icon: IconSettings, id: "settings", label: "Settings" },
                ]}
                label="Catalog content tabs"
                listRef={null}
                onChange={() => undefined}
                onScrollLeft={() => undefined}
                onScrollRight={() => undefined}
              />
            </div>
          </Example>
        </div>
      </Section>

      <Section
        description="Settings rows combine labels, descriptions, lifecycle status badges, and aligned controls."
        id="settings-rows"
        title="Settings Rows"
        visible={isVisible("settings-rows")}
      >
        <div className={styles.exampleGrid}>
          <Example title="Preference Rows" wide>
            <SettingsPanel label="Catalog settings rows">
              <SettingsRow
                description="Controls whether direct messages and mentions create local alerts."
                label="Message alerts"
                status="Live"
              >
                <SettingsToggle checked={settingsAlerts} label="Message alerts" onChange={setSettingsAlerts} />
              </SettingsRow>
              <SettingsRow
                description="Changes the density used by message and channel surfaces."
                label="Interface density"
                status="Review"
              >
                <SettingsSelect onChange={(event) => setSettingsDensity(event.target.value)} value={settingsDensity}>
                  <option value="comfortable">Comfortable</option>
                  <option value="compact">Compact</option>
                </SettingsSelect>
              </SettingsRow>
              <SettingsRow
                description="Current delivery route for server-backed encrypted messages."
                label="Delivery route"
                status="Locked"
              >
                <SettingsValue>Server relay</SettingsValue>
              </SettingsRow>
              <SettingsRow
                description="Manual action pattern used by settings and internal operations."
                label="Cache controls"
                status="Dev only"
              >
                <SettingsButton onClick={() => undefined}>Refresh</SettingsButton>
              </SettingsRow>
            </SettingsPanel>
          </Example>
        </div>
      </Section>

      <Section
        description="Hub surfaces cover the reusable server/contact toolbar, list and card item layouts, item actions, and bulk action bar."
        id="hub-surfaces"
        title="Hub Surfaces"
        visible={isVisible("hub-surfaces")}
      >
        <div className={styles.exampleGrid}>
          <Example title="Toolbar, Surface, And Bulk Actions" wide>
            <HubToolbar
              actions={
                <Button icon={<IconPlus aria-hidden="true" />} variant="primary">
                  Create
                </Button>
              }
              layout={hubLayout}
              mutedOnly={hubMutedOnly}
              onLayoutChange={setHubLayout}
              onMutedChange={() => setHubMutedOnly((value) => !value)}
              onPinnedChange={() => setHubPinnedOnly((value) => !value)}
              onSearchChange={setHubSearch}
              onUnreadChange={() => setHubUnreadOnly((value) => !value)}
              pinnedOnly={hubPinnedOnly}
              search={hubSearch}
              searchLabel="Search servers"
              unreadOnly={hubUnreadOnly}
            />
            <HubSurface
              items={hubItems}
              layout={hubLayout}
              noun="server"
              onOpen={() => undefined}
              onToggleSelected={toggleHubSelected}
              renderActions={(item) => (
                <ItemActions
                  busy={false}
                  destructiveLabel="Leave"
                  messageAction={{ label: "Open", onClick: () => undefined }}
                  muted={item.muted}
                  onDestructive={() => undefined}
                  onToggleMuted={() => undefined}
                  onTogglePinned={() => undefined}
                  pinned={item.pinned}
                />
              )}
              renderBadges={(item) => <Badge tone="muted">{item.id === "atlas" ? "12 channels" : "4 channels"}</Badge>}
              selectedIds={hubSelectedIds}
              selecting={hubSelectedIds.size > 0}
            />
            <BulkActions
              busy={false}
              destructiveLabel="Leave"
              onDestructive={() => undefined}
              onDone={() => setHubSelectedIds(new Set())}
              onMute={() => undefined}
              onPin={() => undefined}
              onUnmute={() => undefined}
              onUnpin={() => undefined}
              selectedCount={hubSelectedIds.size}
            />
          </Example>
        </div>
      </Section>

      <Section
        description="Workspace rows document server identity, member cards, and voice participant states from the server workspace."
        id="workspace-rows"
        title="Workspace Rows"
        visible={isVisible("workspace-rows")}
      >
        <div className={styles.exampleGrid}>
          <Example title="Server Identity">
            <div className={styles.workspaceIconSample}>
              <WorkspaceIcon name="Atlas Team" />
              <div>
                <h3>Atlas Team</h3>
                <p>Server icon treatment used by workspace surfaces.</p>
              </div>
            </div>
          </Example>

          <Example title="Members" wide>
            <div className={styles.workspaceRowStack}>
              {members.map((member, index) => (
                <MemberCard
                  authorHandle={authorHandle}
                  authorLabel={authorLabel}
                  current={index === 0}
                  formatTimestamp={formatTimestamp}
                  key={member.identityId}
                  member={member}
                />
              ))}
            </div>
          </Example>

          <Example title="Voice Participants" wide>
            <div className={styles.workspaceRowStack}>
              <VoiceParticipantRow
                authorHandle={authorHandle}
                authorLabel={authorLabel}
                identityId="usr_aline"
                speaking
              />
              <VoiceParticipantRow
                authorHandle={authorHandle}
                authorLabel={authorLabel}
                identityId="usr_mara"
                speaking={false}
              />
            </div>
          </Example>
        </div>
      </Section>

      <Section
        description="Contact cards cover inbound requests and discovery results used by the friends hub."
        id="contacts"
        title="Contacts"
        visible={isVisible("contacts")}
      >
        <div className={styles.exampleGrid}>
          <Example title="Requests" wide>
            <ContactRequestSection
              busyRequestId={null}
              formatDateTime={formatTimestamp}
              identityId="usr_aline"
              identityLabel={(identityId) => authorLabel(identityId)}
              kind="inbound"
              onTransition={() => Promise.resolve()}
              personas={[]}
              requests={contactRequests}
            />
          </Example>

          <Example title="Discovery" wide>
            <Panel>
              <ContactDiscoveryResults
                onSendFriendRequest={() => undefined}
                sendBusyIdentityId={null}
                shortIdentity={shortIdentity}
                users={discoveredContacts}
              />
            </Panel>
          </Example>
        </div>
      </Section>
    </>
  );
}
