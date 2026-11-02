package dev.fivesaw.sawbyte;

import net.fabricmc.api.ClientModInitializer;
import net.fabricmc.fabric.api.client.event.lifecycle.v1.ClientTickEvents;
import net.minecraft.client.Minecraft;
import java.io.BufferedWriter;
import java.io.IOException;
import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;
import java.util.Locale;

/** Optional replay bridge. Inert unless explicitly launched with sawbyte.tape. */
public final class SawbyteBridge implements ClientModInitializer {
    private record Frame(int buttons, double yaw, double pitch) {}
    private Frame[] frames;
    private BufferedWriter trace;
    private int index;
    private int settle = 100;
    private boolean started;
    private boolean finished;
    private boolean applied;
    private final boolean exitOnComplete = Boolean.getBoolean("sawbyte.exit");

    @Override public void onInitializeClient() {
        String filename = System.getProperty("sawbyte.tape");
        if (filename == null) return;
        try {
            frames = load(Path.of(filename));
            trace = Files.newBufferedWriter(Path.of(System.getProperty("sawbyte.trace", "sawbyte-reference.csv")));
            trace.write("tick,x,y,z,yaw,pitch,on_ground,chunks\n");
        } catch (IOException error) {
            throw new IllegalStateException("Cannot load Sawbyte replay", error);
        }
        ClientTickEvents.START_CLIENT_TICK.register(this::beforeTick);
        ClientTickEvents.END_CLIENT_TICK.register(this::afterTick);
    }

    private static Frame[] load(Path path) throws IOException {
        long size = Files.size(path);
        if (size < 30 || size > 12 + 12000 * 18) throw new IOException("Invalid tape size");
        ByteBuffer data = ByteBuffer.wrap(Files.readAllBytes(path)).order(ByteOrder.LITTLE_ENDIAN);
        byte[] magic = new byte[8];
        data.get(magic);
        int count = data.getInt();
        if (!Arrays.equals(magic, "SBIF0001".getBytes(StandardCharsets.US_ASCII))
                || count < 1 || count > 12000 || size != 12L + count * 18L) {
            throw new IOException("Invalid tape header");
        }
        Frame[] result = new Frame[count];
        for (int i = 0; i < count; i++) {
            int buttons = Short.toUnsignedInt(data.getShort());
            double yaw = data.getDouble();
            double pitch = data.getDouble();
            if ((buttons & ~0x7f) != 0 || !Double.isFinite(yaw) || !Double.isFinite(pitch)
                    || Math.abs(yaw) > 360 || Math.abs(pitch) > 180) {
                throw new IOException("Unsupported or invalid frame");
            }
            result[i] = new Frame(buttons, yaw, pitch);
        }
        Frame last = result[count - 1];
        if (last.buttons != 0 || last.yaw != 0 || last.pitch != 0) {
            throw new IOException("Tape must end with neutral input");
        }
        return result;
    }

    private void beforeTick(Minecraft client) {
        applied = false;
        if (finished) return;
        if (client.player == null || client.level == null) {
            if (started) finish(client, "disconnected during replay");
            return;
        }
        if (client.screen != null) {
            if (started) finish(client, "screen opened: manual handoff");
            return;
        }
        if (!client.level.hasChunkAt(client.player.blockPosition())) return;
        if (client.player.isDeadOrDying()) { finish(client, "player died"); return; }
        if (settle > 0) { settle--; keys(client, 0); return; }
        if (!started) System.out.println("SAWBYTE_REPLAY_STARTED");
        started = true;
        Frame frame = frames[index];
        keys(client, frame.buttons);
        // Degrees are already shaped by Sawbyte's input policy in the shared tape.
        // No positions or movement packets are written by this bridge.
        client.player.setYRot(client.player.getYRot() + (float) frame.yaw);
        client.player.setXRot(Math.clamp(client.player.getXRot() + (float) frame.pitch, -90f, 90f));
        applied = true;
    }

    private void afterTick(Minecraft client) {
        if (!applied || finished || client.player == null) return;
        try {
            trace.write(String.format(Locale.ROOT, "%d,%.9f,%.9f,%.9f,%.6f,%.6f,%s,1%n",
                index, client.player.getX(), client.player.getY(), client.player.getZ(),
                client.player.getYRot(), client.player.getXRot(), client.player.onGround()));
            index++;
            if (index == frames.length) finish(client, null);
        } catch (IOException error) {
            finish(client, "trace write failed");
        }
    }

    private static void keys(Minecraft client, int bits) {
        client.options.keyUp.setDown((bits & 1) != 0);
        client.options.keyDown.setDown((bits & 2) != 0);
        client.options.keyLeft.setDown((bits & 4) != 0);
        client.options.keyRight.setDown((bits & 8) != 0);
        client.options.keyJump.setDown((bits & 16) != 0);
        client.options.keyShift.setDown((bits & 32) != 0);
        client.options.keySprint.setDown((bits & 64) != 0);
    }

    private void finish(Minecraft client, String error) {
        if (finished) return;
        finished = true;
        keys(client, 0);
        try { trace.close(); } catch (IOException failure) { error = "trace close failed"; }
        if (error == null) System.out.println("SAWBYTE_REPLAY_COMPLETE frames=" + index);
        else System.err.println("SAWBYTE_REPLAY_FAILED " + error);
        if (exitOnComplete) client.stop();
    }
}
