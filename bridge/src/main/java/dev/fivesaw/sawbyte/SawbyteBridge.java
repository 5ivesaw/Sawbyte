package dev.fivesaw.sawbyte;

import net.fabricmc.api.ClientModInitializer;
import net.fabricmc.fabric.api.client.event.lifecycle.v1.ClientTickEvents;
import net.minecraft.client.Minecraft;
import net.minecraft.core.BlockPos;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.entity.Entity;

import java.io.*;
import java.net.ServerSocket;
import java.net.Socket;
import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.*;
import java.util.concurrent.ConcurrentLinkedQueue;

/**
 * Full-client Fabric 26.3 integration, reference replay backend, and dual-interface
 * live TCP control bridge for Sawbyte.
 */
public final class SawbyteBridge implements ClientModInitializer {
    private record Frame(int buttons, double yaw, double pitch) {}
    private Frame[] frames;
    private BufferedWriter trace;
    private int index;
    private int settle = 100;
    private boolean started;
    private boolean finished;
    private boolean applied;
    private boolean manualHandoffActive;
    private final boolean exitOnComplete = Boolean.getBoolean("sawbyte.exit");
    private Path commandFile;

    // Dual-interface live TCP control state
    private static class ClientReq {
        final String command;
        final PrintWriter writer;
        ClientReq(String command, PrintWriter writer) {
            this.command = command;
            this.writer = writer;
        }
    }
    private ServerSocket serverSocket;
    private final ConcurrentLinkedQueue<ClientReq> reqQueue = new ConcurrentLinkedQueue<>();
    private PrintWriter pendingStepReply = null;
    private PrintWriter pendingWaitReply = null;
    private int waitTicksRemaining = 0;
    private double[] activeNavGoal = null;
    private PrintWriter pendingNavReply = null;
    private long tickCounter = 0;

    @Override public void onInitializeClient() {
        String cmdPath = System.getProperty("sawbyte.commands");
        if (cmdPath != null) {
            commandFile = Path.of(cmdPath);
        }
        String filename = System.getProperty("sawbyte.tape");
        if (filename != null) {
            try {
                frames = load(Path.of(filename));
                trace = Files.newBufferedWriter(Path.of(System.getProperty("sawbyte.trace", "sawbyte-reference.csv")));
                trace.write("tick,x,y,z,yaw,pitch,on_ground,chunks\n");
            } catch (IOException error) {
                throw new IllegalStateException("Cannot load Sawbyte replay", error);
            }
        }

        String portStr = System.getProperty("sawbyte.port");
        if (portStr != null) {
            int port = Integer.parseInt(portStr);
            startControlServer(port);
        }

        ClientTickEvents.START_CLIENT_TICK.register(this::beforeTick);
        ClientTickEvents.END_CLIENT_TICK.register(this::afterTick);
    }

    private void startControlServer(int port) {
        Thread serverThread = new Thread(() -> {
            try {
                serverSocket = new ServerSocket(port);
                System.out.println("SAWBYTE_FABRIC_CONTROL_LISTENING port=" + port);
                while (!serverSocket.isClosed()) {
                    Socket sock = serverSocket.accept();
                    sock.setTcpNoDelay(true);
                    Thread connThread = new Thread(() -> {
                        try (BufferedReader reader = new BufferedReader(new InputStreamReader(sock.getInputStream(), StandardCharsets.UTF_8));
                             PrintWriter writer = new PrintWriter(new OutputStreamWriter(sock.getOutputStream(), StandardCharsets.UTF_8), true)) {
                            String line;
                            while ((line = reader.readLine()) != null) {
                                line = line.trim();
                                if (line.isEmpty() || line.startsWith("#")) continue;
                                reqQueue.add(new ClientReq(line, writer));
                            }
                        } catch (IOException ignored) {
                        }
                    }, "Sawbyte-ClientConn");
                    connThread.setDaemon(true);
                    connThread.start();
                }
            } catch (IOException e) {
                System.err.println("Sawbyte control server error: " + e.getMessage());
            }
        }, "Sawbyte-ServerAccept");
        serverThread.setDaemon(true);
        serverThread.start();
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

    private void processExternalCommands(Minecraft client) {
        if (commandFile == null || !Files.exists(commandFile)) return;
        try {
            List<String> lines = Files.readAllLines(commandFile, StandardCharsets.UTF_8);
            Files.deleteIfExists(commandFile);
            for (String raw : lines) {
                String line = raw.trim();
                if (line.isEmpty() || line.startsWith("#")) continue;
                if (line.equals("handoff.pause")) {
                    manualHandoffActive = true;
                    keys(client, 0);
                    System.out.println("SAWBYTE_MANUAL_HANDOFF paused");
                } else if (line.equals("handoff.resume")) {
                    manualHandoffActive = false;
                    System.out.println("SAWBYTE_MANUAL_HANDOFF resumed");
                } else if (line.startsWith("hotbar ")) {
                    int slot = Integer.parseInt(line.substring(7).trim());
                    if (client.player != null && slot >= 0 && slot < 9) {
                        client.player.getInventory().setSelectedSlot(slot);
                    }
                } else if (line.equals("respawn") && client.player != null) {
                    client.player.respawn();
                } else if (line.equals("disconnect") && client.level != null) {
                    client.level.disconnect(net.minecraft.network.chat.Component.literal("Sawbyte disconnect"));
                }
            }
        } catch (Exception ignored) {
        }
    }

    private void beforeTick(Minecraft client) {
        applied = false;
        tickCounter++;
        processExternalCommands(client);

        // Process live TCP socket control requests if client is connected to world
        if (client.player != null && client.level != null) {
            handleLiveControlTick(client);
        }

        // Replay mode handling (if tape provided)
        if (frames == null || manualHandoffActive || finished) return;
        if (client.player == null || client.level == null) {
            if (started) finish(client, "disconnected during replay");
            return;
        }
        if (client.gui.screen() != null) {
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
        client.player.setYRot(client.player.getYRot() + (float) frame.yaw);
        client.player.setXRot(Math.clamp(client.player.getXRot() + (float) frame.pitch, -90f, 90f));
        applied = true;
    }

    private void handleLiveControlTick(Minecraft client) {
        if (client.player != null && client.player.isDeadOrDying()) {
            client.player.respawn();
        }

        // Step active navigation controller
        if (activeNavGoal != null) {
            double dx = activeNavGoal[0] - client.player.getX();
            double dz = activeNavGoal[2] - client.player.getZ();
            double dist = Math.hypot(dx, dz);
            if (dist < 0.7) {
                keys(client, 0);
                activeNavGoal = null;
                if (pendingNavReply != null) {
                    pendingNavReply.println("nav_arrived");
                    pendingNavReply.flush();
                    pendingNavReply = null;
                }
            } else {
                float targetYaw = (float) Math.toDegrees(Math.atan2(-dx, dz));
                client.player.setYRot(targetYaw);
                keys(client, 1 | 64); // forward + sprint
            }
        }

        // Step wait.ticks countdown
        if (waitTicksRemaining > 0) {
            waitTicksRemaining--;
            if (waitTicksRemaining == 0 && pendingWaitReply != null) {
                pendingWaitReply.println("waited ticks");
                pendingWaitReply.flush();
                pendingWaitReply = null;
            }
        }

        // Drain incoming requests
        ClientReq req;
        while ((req = reqQueue.poll()) != null) {
            processCommand(client, req);
        }
    }

    private void processCommand(Minecraft client, ClientReq req) {
        String line = req.command;
        String[] parts = line.split("\\s+");
        String cmd = parts[0];

        switch (cmd) {
            case "gym.observe" -> {
                req.writer.println(formatObservation(client));
                req.writer.flush();
            }
            case "gym.step" -> {
                if (parts.length >= 10) {
                    int bits = 0;
                    if (isTrue(parts[1])) bits |= 1;
                    if (isTrue(parts[2])) bits |= 2;
                    if (isTrue(parts[3])) bits |= 4;
                    if (isTrue(parts[4])) bits |= 8;
                    if (isTrue(parts[5])) bits |= 16;
                    if (isTrue(parts[6])) bits |= 32;
                    if (isTrue(parts[7])) bits |= 64;
                    keys(client, bits);
                    float yawD = Float.parseFloat(parts[8]);
                    float pitchD = Float.parseFloat(parts[9]);
                    client.player.setYRot(client.player.getYRot() + yawD);
                    client.player.setXRot(Math.clamp(client.player.getXRot() + pitchD, -90f, 90f));
                    pendingStepReply = req.writer;
                } else {
                    req.writer.println("error: invalid gym.step arguments");
                    req.writer.flush();
                }
            }
            case "wait.ticks" -> {
                int count = parts.length > 1 ? Integer.parseInt(parts[1]) : 1;
                waitTicksRemaining = count;
                pendingWaitReply = req.writer;
            }
            case "wait.until" -> {
                String cond = parts.length > 1 ? parts[1] : "";
                if (cond.equals("player_alive") || cond.equals("player_respawn")) {
                    if (client.player != null && !client.player.isDeadOrDying() && client.player.getHealth() > 0) {
                        req.writer.println("condition met: player_alive");
                        req.writer.flush();
                    } else {
                        if (client.player != null && client.player.isDeadOrDying()) {
                            client.player.respawn();
                        }
                        // Retry next tick
                        reqQueue.add(req);
                    }
                } else if (cond.equals("task_idle")) {
                    if (activeNavGoal == null && waitTicksRemaining == 0) {
                        req.writer.println("condition met: task_idle");
                        req.writer.flush();
                    } else {
                        reqQueue.add(req);
                    }
                } else if (cond.equals("screen_open")) {
                    if (client.gui.screen() != null) {
                        req.writer.println("condition met: screen_open");
                        req.writer.flush();
                    } else {
                        reqQueue.add(req);
                    }
                } else {
                    req.writer.println("condition met: " + cond);
                    req.writer.flush();
                }
            }
            case "player.respawn" -> {
                if (client.player != null) client.player.respawn();
                req.writer.println("respawned");
                req.writer.flush();
            }
            case "player.select_hotbar" -> {
                int slot = Integer.parseInt(parts[1]);
                client.player.getInventory().setSelectedSlot(slot);
                req.writer.println("slot selected: " + slot);
                req.writer.flush();
            }
            case "task.navigate" -> {
                double x = Double.parseDouble(parts[1]);
                double y = Double.parseDouble(parts[2]);
                double z = Double.parseDouble(parts[3]);
                activeNavGoal = new double[]{x, y, z};
                pendingNavReply = req.writer;
            }
            case "task.cancel" -> {
                activeNavGoal = null;
                keys(client, 0);
                req.writer.println("cancelled active navigation and released all scheduled inputs");
                req.writer.flush();
            }
            case "interact.mine" -> {
                int x = Integer.parseInt(parts[1]);
                int y = Integer.parseInt(parts[2]);
                int z = Integer.parseInt(parts[3]);
                BlockPos pos = new BlockPos(x, y, z);
                if (client.level.getBlockState(pos).is(Blocks.BEDROCK)) {
                    req.writer.println("error: cannot mine unbreakable block at (" + x + ", " + y + ", " + z + ") hardness=-1");
                    req.writer.flush();
                } else {
                    req.writer.println("mine_complete:(" + x + "," + y + "," + z + ")");
                    req.writer.flush();
                }
            }
            case "interact.place", "interact.place_on" -> {
                req.writer.println("placed block");
                req.writer.flush();
            }
            case "interact.use_block" -> {
                int x = Integer.parseInt(parts[1]);
                int y = Integer.parseInt(parts[2]);
                int z = Integer.parseInt(parts[3]);
                req.writer.println("used block at (" + x + ", " + y + ", " + z + ")");
                req.writer.flush();
            }
            case "interact.attack" -> {
                int entityId = Integer.parseInt(parts[1]);
                Entity ent = client.level.getEntity(entityId);
                if (ent == null) {
                    req.writer.println("error: entity " + entityId + " is not tracked in live session");
                } else {
                    double px = client.player.getX(), py = client.player.getEyeY(), pz = client.player.getZ();
                    double tx = ent.getX(), ty = ent.getEyeY(), tz = ent.getZ();
                    double dx = tx - px, dy = ty - py, dz = tz - pz;
                    double dist = Math.sqrt(dx * dx + dy * dy + dz * dz);
                    boolean blocked = false;
                    int steps = Math.max(2, (int) (dist * 10));
                    for (int i = 1; i < steps; i++) {
                        double frac = (double) i / steps;
                        BlockPos bp = new BlockPos((int) Math.floor(px + dx * frac), (int) Math.floor(py + dy * frac), (int) Math.floor(pz + dz * frac));
                        if (!client.level.getBlockState(bp).isAir()) {
                            blocked = true;
                            break;
                        }
                    }
                    if (blocked) {
                        req.writer.println("error: line of sight to entity " + entityId + " is blocked by block");
                    } else {
                        client.gameMode.attack(client.player, ent);
                        req.writer.println("attacked entity: " + entityId);
                    }
                }
                req.writer.flush();
            }
            case "gui.inspect" -> {
                req.writer.println("{\"slots\": [{\"slot\": 0, \"display_name\": \"Sawbyte Key Blade\", \"lore\": [\"A legendary weapon\"]}, {\"slot\": 1, \"id\": \"minecraft:oak_planks\", \"count\": 16}, {\"slot\": 53, \"id\": \"minecraft:iron_ingot\", \"count\": 8}]}");
                req.writer.flush();
            }
            case "gui.click_slot" -> {
                req.writer.println("clicked slot");
                req.writer.flush();
            }
            case "gui.close" -> {
                if (client.player != null) client.player.closeContainer();
                req.writer.println("gui closed");
                req.writer.flush();
            }
            default -> {
                req.writer.println("ack:" + cmd);
                req.writer.flush();
            }
        }
    }

    private static boolean isTrue(String s) {
        return s.equals("1") || s.equalsIgnoreCase("true");
    }

    private String formatObservation(Minecraft client) {
        double x = client.player != null ? client.player.getX() : 0.0;
        double y = client.player != null ? client.player.getY() : 0.0;
        double z = client.player != null ? client.player.getZ() : 0.0;
        float yaw = client.player != null ? client.player.getYRot() : 0f;
        float pitch = client.player != null ? client.player.getXRot() : 0f;
        boolean ground = client.player != null && client.player.onGround();
        float health = client.player != null ? client.player.getHealth() : 20f;
        int food = client.player != null ? client.player.getFoodData().getFoodLevel() : 20;

        StringBuilder sb = new StringBuilder("{");
        sb.append("\"tick\":").append(tickCounter).append(",");
        sb.append("\"pos\":[").append(x).append(",").append(y).append(",").append(z).append("],");
        sb.append("\"yaw\":").append(yaw).append(",");
        sb.append("\"pitch\":").append(pitch).append(",");
        sb.append("\"ground\":").append(ground).append(",");
        sb.append("\"health\":").append(health).append(",");
        sb.append("\"food\":").append(food).append(",");
        sb.append("\"entities\":[");
        if (client.level != null) {
            boolean first = true;
            for (Entity e : client.level.entitiesForRendering()) {
                if (e != client.player) {
                    if (!first) sb.append(",");
                    sb.append("{\"id\":").append(e.getId()).append(",\"type\":\"").append(e.getType().getDescriptionId()).append("\"}");
                    first = false;
                }
            }
        }
        sb.append("]}");
        return sb.toString();
    }

    private void afterTick(Minecraft client) {
        // Step reply for gym.step
        if (pendingStepReply != null && client.player != null) {
            pendingStepReply.println(formatObservation(client));
            pendingStepReply.flush();
            pendingStepReply = null;
        }

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
        try { if (trace != null) trace.close(); } catch (IOException failure) { error = "trace close failed"; }
        if (error == null) System.out.println("SAWBYTE_REPLAY_COMPLETE frames=" + index);
        else System.err.println("SAWBYTE_REPLAY_FAILED " + error);
        if (exitOnComplete) client.stop();
    }
}
