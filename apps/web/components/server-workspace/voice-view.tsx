import { IconInfoCircle, IconMicrophone, IconVolume } from "@tabler/icons-react";

import { ChannelRail } from "@/components/chat/channel-rail";
import { Button } from "@/components/ui/button";

import { VoiceChannelButton } from "./voice-channel-button";
import { VoiceParticipantRow } from "./voice-participant-row";
import type { VoiceChannel } from "./types";

import styles from "@/app/surfaces.module.css";

type VoiceViewProps = {
  activeChannel: VoiceChannel | null;
  authorHandle: (identityId: string) => string;
  authorLabel: (identityId: string) => string;
  channels: VoiceChannel[];
  onSelectChannel: (channelId: string) => void;
};

export function VoiceView({
  activeChannel,
  authorHandle,
  authorLabel,
  channels,
  onSelectChannel,
}: VoiceViewProps) {
  return (
    <section className={styles.chatGrid} aria-label="Server voice">
      <ChannelRail aria-label="Voice channels" title="Voice channels">
        {channels.map((channel) => (
          <VoiceChannelButton
            active={channel.id === activeChannel?.id}
            channel={channel}
            key={channel.id}
            onSelect={onSelectChannel}
          />
        ))}
      </ChannelRail>

      <article className={`${styles.chatPanel} ${styles.voicePanel}`}>
        <header className={styles.chatHeader}>
          <div>
            <p className={styles.serverSectionLabel}>Voice channel</p>
            <h3>
              <IconVolume className={styles.icon} aria-hidden="true" />
              {activeChannel?.name ?? "No voice channel"}
            </h3>
            <p className={styles.serverMeta}>{activeChannel?.description ?? "Select a voice channel."}</p>
          </div>
          <Button disabled icon={<IconMicrophone className={styles.icon} aria-hidden="true" />} variant="primary">
            Join voice
          </Button>
        </header>

        <div className={styles.voiceStage}>
          <section className={styles.voiceStatusCard} aria-label="Voice session state">
            <div className={styles.panelHeader}>
              <h3>Session</h3>
              <span>{activeChannel?.participantIds.length ?? 0} connected</span>
            </div>
            <p className={styles.meta}>
              {activeChannel && activeChannel.participantIds.length > 0
                ? `${authorLabel(activeChannel.speakerId ?? activeChannel.participantIds[0])} is currently active.`
                : "This room is idle in the seeded preview."}
            </p>
            <div className={styles.voiceMeter} aria-hidden="true">
              <span className={activeChannel?.participantIds.length ? styles.voiceMeterActive : ""} />
              <span className={activeChannel?.participantIds.length ? styles.voiceMeterActive : ""} />
              <span />
              <span />
            </div>
          </section>

          <section className={styles.voiceStatusCard} aria-label="Voice participants">
            <div className={styles.panelHeader}>
              <h3>Participants</h3>
              <span>{activeChannel?.participantIds.length ?? 0}</span>
            </div>
            {activeChannel && activeChannel.participantIds.length > 0 ? (
              <div className={styles.voiceParticipantList}>
                {activeChannel.participantIds.map((participantId) => (
                  <VoiceParticipantRow
                    authorHandle={authorHandle}
                    authorLabel={authorLabel}
                    identityId={participantId}
                    key={participantId}
                    speaking={participantId === activeChannel.speakerId}
                  />
                ))}
              </div>
            ) : (
              <div className={styles.voiceEmptyState}>
                <IconVolume className={styles.icon} aria-hidden="true" />
                <p>No one is connected to this voice channel.</p>
              </div>
            )}
          </section>

          <section className={styles.voiceStatusCard} aria-label="Voice controls">
            <div className={styles.panelHeader}>
              <h3>Controls</h3>
              <span>Preview</span>
            </div>
            <div className={styles.voiceControlGrid}>
              <Button disabled icon={<IconMicrophone className={styles.icon} aria-hidden="true" />}>
                Mute
              </Button>
              <Button disabled icon={<IconVolume className={styles.icon} aria-hidden="true" />}>
                Deafen
              </Button>
            </div>
            <p className={styles.meta}>Voice controls are disabled until local voice runtime bindings are available.</p>
          </section>
        </div>

        <div className={styles.composerLocked}>
          <IconInfoCircle className={styles.icon} aria-hidden="true" />
          Activate a local testing profile and voice runtime to join channels.
        </div>
      </article>
    </section>
  );
}
