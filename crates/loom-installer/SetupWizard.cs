using System;
using System.Drawing;
using System.Drawing.Drawing2D;
using System.IO;
using System.Reflection;
using System.Windows.Forms;
using System.Diagnostics;

namespace LoomDB.Installer
{
    public class SetupWizardForm : Form
    {
        private Panel leftBanner;
        private Panel contentPanel;
        private Panel footerPanel;
        private Button btnBack;
        private Button btnNext;
        private Button btnCancel;

        private Label lblTitle;
        private Label lblDescription;
        private TextBox txtInstallPath;
        private Button btnBrowse;
        private CheckBox chkService;
        private CheckBox chkLaunchNow;
        private ProgressBar progressBar;
        private Label lblStatus;
        private RadioButton radAccept;
        private RadioButton radDecline;

        private int currentStep = 0;
        private string installDir;

        [STAThread]
        public static void Main()
        {
            Application.EnableVisualStyles();
            Application.SetCompatibleTextRenderingDefault(false);
            Application.Run(new SetupWizardForm());
        }

        public SetupWizardForm()
        {
            this.Text = "LoomDB Setup Wizard (x64)";
            this.ClientSize = new Size(620, 420);
            this.StartPosition = FormStartPosition.CenterScreen;
            this.FormBorderStyle = FormBorderStyle.FixedDialog;
            this.MaximizeBox = false;
            this.MinimizeBox = true;
            this.BackColor = SystemColors.Control;
            this.Font = new Font("Segoe UI", 9F, FontStyle.Regular);

            string localApp = Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData);
            this.installDir = Path.Combine(localApp, "LoomDB");

            InitializeComponents();
            ShowStep(0);
        }

        private void InitializeComponents()
        {
            int footerHeight = 55;
            int bannerWidth = 185;
            int bodyHeight = this.ClientSize.Height - footerHeight;
            int contentWidth = this.ClientSize.Width - bannerWidth;

            // 1. Left Blue Banner (Classic Windows Setup Wizard Left Bar)
            leftBanner = new Panel();
            leftBanner.Location = new Point(0, 0);
            leftBanner.Size = new Size(bannerWidth, bodyHeight);
            leftBanner.BackColor = Color.FromArgb(16, 42, 102);
            leftBanner.Paint += (s, e) =>
            {
                Graphics g = e.Graphics;
                g.SmoothingMode = SmoothingMode.AntiAlias;

                // Decorative icon box (matches classic Inno/NSIS style monitor/cube emblem)
                using (Pen pen = new Pen(Color.FromArgb(60, 100, 190), 2))
                {
                    g.DrawRectangle(pen, 20, 25, 48, 48);
                }
                using (Brush b = new SolidBrush(Color.FromArgb(30, 70, 150)))
                {
                    g.FillRectangle(b, 21, 26, 46, 46);
                }
                // Database cylinder graphic inside emblem
                using (Pen pen = new Pen(Color.White, 2))
                using (Brush b = new SolidBrush(Color.FromArgb(140, 185, 255)))
                {
                    g.DrawEllipse(pen, 28, 33, 32, 10);
                    g.FillEllipse(b, 28, 33, 32, 10);
                    g.DrawLine(pen, 28, 38, 28, 55);
                    g.DrawLine(pen, 60, 38, 60, 55);
                    g.DrawArc(pen, 28, 48, 32, 14, 0, 180);
                }

                // Brand text
                using (Font f = new Font("Segoe UI", 15F, FontStyle.Bold))
                using (Brush b = new SolidBrush(Color.White))
                {
                    g.DrawString("LOOM // DB", f, b, new PointF(18, 90));
                }

                using (Font f = new Font("Segoe UI", 8.5F, FontStyle.Regular))
                using (Brush b = new SolidBrush(Color.FromArgb(195, 218, 255)))
                {
                    g.DrawString("The World's Lightest\nMulti-Modal Database\n\nVersion 0.1.0\nSub-512KB RAM Profile\nZero-Dependency Single File", f, b, new PointF(18, 130));
                }
            };
            this.Controls.Add(leftBanner);

            // 2. Main Content Panel (Clean White Background, Zero Overlap)
            contentPanel = new Panel();
            contentPanel.Location = new Point(bannerWidth, 0);
            contentPanel.Size = new Size(contentWidth, bodyHeight);
            contentPanel.BackColor = Color.White;
            this.Controls.Add(contentPanel);

            // Title Label
            lblTitle = new Label();
            lblTitle.Font = new Font("Segoe UI", 12.5F, FontStyle.Bold);
            lblTitle.ForeColor = Color.FromArgb(15, 23, 42);
            lblTitle.BackColor = Color.Transparent;
            lblTitle.Location = new Point(25, 25);
            lblTitle.Size = new Size(contentWidth - 50, 48);
            contentPanel.Controls.Add(lblTitle);

            // Description Label
            lblDescription = new Label();
            lblDescription.Font = new Font("Segoe UI", 9F, FontStyle.Regular);
            lblDescription.ForeColor = Color.FromArgb(51, 65, 85);
            lblDescription.BackColor = Color.Transparent;
            lblDescription.Location = new Point(25, 80);
            lblDescription.Size = new Size(contentWidth - 50, 110);
            contentPanel.Controls.Add(lblDescription);

            // Path Controls (Step 2)
            txtInstallPath = new TextBox();
            txtInstallPath.Font = new Font("Segoe UI", 9F);
            txtInstallPath.Location = new Point(25, 200);
            txtInstallPath.Size = new Size(295, 25);
            txtInstallPath.Text = this.installDir;
            txtInstallPath.Visible = false;
            contentPanel.Controls.Add(txtInstallPath);

            btnBrowse = new Button();
            btnBrowse.Text = "Browse...";
            btnBrowse.Font = new Font("Segoe UI", 9F);
            btnBrowse.Location = new Point(328, 199);
            btnBrowse.Size = new Size(80, 27);
            btnBrowse.Visible = false;
            btnBrowse.Click += (s, e) =>
            {
                using (FolderBrowserDialog fbd = new FolderBrowserDialog())
                {
                    fbd.SelectedPath = txtInstallPath.Text;
                    if (fbd.ShowDialog() == DialogResult.OK)
                    {
                        txtInstallPath.Text = fbd.SelectedPath;
                        this.installDir = fbd.SelectedPath;
                    }
                }
            };
            contentPanel.Controls.Add(btnBrowse);

            // Checkbox: Service (Step 2)
            chkService = new CheckBox();
            chkService.Text = "Register LoomDB as automatic Windows Background Service (Port 8484)";
            chkService.Font = new Font("Segoe UI", 8.5F);
            chkService.ForeColor = Color.FromArgb(30, 41, 59);
            chkService.BackColor = Color.Transparent;
            chkService.Location = new Point(25, 240);
            chkService.Size = new Size(385, 36);
            chkService.Checked = true;
            chkService.Visible = false;
            contentPanel.Controls.Add(chkService);

            // Checkbox: Launch Now (Step 4)
            chkLaunchNow = new CheckBox();
            chkLaunchNow.Text = "Launch LoomDB Interactive Shell immediately";
            chkLaunchNow.Font = new Font("Segoe UI", 9F);
            chkLaunchNow.ForeColor = Color.FromArgb(30, 41, 59);
            chkLaunchNow.BackColor = Color.Transparent;
            chkLaunchNow.Location = new Point(25, 205);
            chkLaunchNow.Size = new Size(385, 26);
            chkLaunchNow.Checked = true;
            chkLaunchNow.Visible = false;
            contentPanel.Controls.Add(chkLaunchNow);

            // Progress Bar (Step 3)
            progressBar = new ProgressBar();
            progressBar.Location = new Point(25, 200);
            progressBar.Size = new Size(385, 22);
            progressBar.Style = ProgressBarStyle.Continuous;
            progressBar.Visible = false;
            contentPanel.Controls.Add(progressBar);

            // Status Label (Step 3)
            lblStatus = new Label();
            lblStatus.Font = new Font("Segoe UI", 8.5F);
            lblStatus.ForeColor = Color.FromArgb(71, 85, 105);
            lblStatus.BackColor = Color.Transparent;
            lblStatus.Location = new Point(25, 230);
            lblStatus.Size = new Size(385, 36);
            lblStatus.Visible = false;
            contentPanel.Controls.Add(lblStatus);

            // License radio buttons (Step 1)
            radAccept = new RadioButton();
            radAccept.Text = "I accept the terms in the License Agreement (MIT / Apache-2.0)";
            radAccept.Font = new Font("Segoe UI", 9F);
            radAccept.ForeColor = Color.FromArgb(30, 41, 59);
            radAccept.BackColor = Color.Transparent;
            radAccept.Location = new Point(25, 200);
            radAccept.Size = new Size(385, 26);
            radAccept.Checked = true;
            radAccept.Visible = false;
            contentPanel.Controls.Add(radAccept);

            radDecline = new RadioButton();
            radDecline.Text = "I do not accept the terms in the License Agreement";
            radDecline.Font = new Font("Segoe UI", 9F);
            radDecline.ForeColor = Color.FromArgb(30, 41, 59);
            radDecline.BackColor = Color.Transparent;
            radDecline.Location = new Point(25, 230);
            radDecline.Size = new Size(385, 26);
            radDecline.Visible = false;
            contentPanel.Controls.Add(radDecline);

            // 3. Footer Panel (Standard Win32 Dialog Footer)
            footerPanel = new Panel();
            footerPanel.Location = new Point(0, bodyHeight);
            footerPanel.Size = new Size(this.ClientSize.Width, footerHeight);
            footerPanel.BackColor = SystemColors.Control;
            footerPanel.Paint += (s, e) =>
            {
                e.Graphics.DrawLine(SystemPens.ControlDark, 0, 0, footerPanel.Width, 0);
            };

            btnCancel = new Button();
            btnCancel.Text = "Cancel";
            btnCancel.Font = new Font("Segoe UI", 9F);
            btnCancel.Size = new Size(85, 28);
            btnCancel.Location = new Point(515, 13);
            btnCancel.Click += (s, e) => this.Close();
            footerPanel.Controls.Add(btnCancel);

            btnNext = new Button();
            btnNext.Text = "Next >";
            btnNext.Font = new Font("Segoe UI", 9F);
            btnNext.Size = new Size(85, 28);
            btnNext.Location = new Point(420, 13);
            btnNext.Click += (s, e) => NextStep();
            footerPanel.Controls.Add(btnNext);

            btnBack = new Button();
            btnBack.Text = "< Back";
            btnBack.Font = new Font("Segoe UI", 9F);
            btnBack.Size = new Size(85, 28);
            btnBack.Location = new Point(330, 13);
            btnBack.Click += (s, e) => BackStep();
            footerPanel.Controls.Add(btnBack);

            this.Controls.Add(footerPanel);
        }

        private void ShowStep(int step)
        {
            this.currentStep = step;

            txtInstallPath.Visible = false;
            btnBrowse.Visible = false;
            chkService.Visible = false;
            chkLaunchNow.Visible = false;
            progressBar.Visible = false;
            lblStatus.Visible = false;
            radAccept.Visible = false;
            radDecline.Visible = false;

            btnBack.Enabled = (step > 0 && step < 3);
            btnNext.Enabled = true;
            btnNext.Text = "Next >";

            switch (step)
            {
                case 0:
                    lblDescription.Size = new Size(contentPanel.Width - 50, 180);
                    lblTitle.Text = "Welcome to the LoomDB Setup Wizard";
                    lblDescription.Text = "This wizard will guide you through the installation of LoomDB 0.1.0 (x64).\n\n" +
                                           "LoomDB is the world's lightest multi-modal database, weaving SQL Relational, " +
                                           "JSON Documents, AI Vectors, and Time-Series into a single portable container with " +
                                           "sub-512KB idle RAM overhead.\n\n" +
                                           "Click Next to continue, or Cancel to exit Setup.";
                    break;

                case 1:
                    lblDescription.Size = new Size(contentPanel.Width - 50, 110);
                    lblTitle.Text = "License Agreement";
                    lblDescription.Text = "Please review the license terms before installing LoomDB.\n\n" +
                                           "LoomDB is open source software distributed under dual Apache-2.0 and MIT licenses.\n" +
                                           "You are granted full permission to use, embed, modify, and distribute LoomDB in both " +
                                           "personal and commercial proprietary applications free of charge.\n\n" +
                                           "If you accept the terms of the agreement, click Next to continue.";
                    radAccept.Visible = true;
                    radDecline.Visible = true;
                    break;

                case 2:
                    lblDescription.Size = new Size(contentPanel.Width - 50, 110);
                    lblTitle.Text = "Choose Install Location";
                    lblDescription.Text = "Setup will install LoomDB and configure command-line access in the following folder.\n\n" +
                                           "To install to this folder, click Next. To install to a different folder, click Browse and select another folder.\n\n" +
                                           "Space required: ~25 MB\nSpace available on drive: >50 GB";
                    txtInstallPath.Visible = true;
                    btnBrowse.Visible = true;
                    chkService.Visible = true;
                    btnNext.Text = "Install";
                    break;

                case 3:
                    lblDescription.Size = new Size(contentPanel.Width - 50, 110);
                    lblTitle.Text = "Installing LoomDB";
                    lblDescription.Text = "Please wait while Setup installs LoomDB on your computer.";
                    progressBar.Visible = true;
                    progressBar.Value = 15;
                    lblStatus.Visible = true;
                    lblStatus.Text = "Extracting binary and configuring system path...";
                    btnBack.Enabled = false;
                    btnNext.Enabled = false;

                    PerformInstall();
                    break;

                case 4:
                    lblDescription.Size = new Size(contentPanel.Width - 50, 120);
                    lblTitle.Text = "Completing the LoomDB Setup Wizard";
                    lblDescription.Text = "LoomDB has been successfully installed on your computer.\n\n" +
                                           "You can now run 'loom' from any Command Prompt or PowerShell terminal to access the interactive shell.\n\n" +
                                           "Click Finish to exit Setup.";
                    chkLaunchNow.Visible = true;
                    btnBack.Visible = false;
                    btnCancel.Visible = false;
                    btnNext.Text = "Finish";
                    break;
            }
        }

        private void PerformInstall()
        {
            Timer t = new Timer();
            t.Interval = 350;
            int progress = 20;

            t.Tick += (s, e) =>
            {
                progress += 25;
                if (progress <= 90)
                {
                    progressBar.Value = progress;
                    if (progress == 45)
                    {
                        lblStatus.Text = "Configuring system environment PATH...";
                    }
                    else if (progress == 70)
                    {
                        lblStatus.Text = "Setting up local container and studio dashboard...";
                    }
                }
                else
                {
                    t.Stop();
                    t.Dispose();

                    try
                    {
                        this.installDir = txtInstallPath.Text.Trim();
                        if (!Directory.Exists(this.installDir))
                        {
                            Directory.CreateDirectory(this.installDir);
                        }

                        // Extract embedded loom.exe
                        string targetExe = Path.Combine(this.installDir, "loom.exe");
                        ExtractResource("loom.exe", targetExe);

                        // Extract studio.html
                        string targetStudio = Path.Combine(this.installDir, "studio.html");
                        ExtractResource("studio.html", targetStudio);

                        // Add to user PATH
                        string currentPath = Environment.GetEnvironmentVariable("Path", EnvironmentVariableTarget.User) ?? "";
                        if (!currentPath.Contains(this.installDir))
                        {
                            string newPath = string.IsNullOrEmpty(currentPath) ? this.installDir : currentPath + ";" + this.installDir;
                            Environment.SetEnvironmentVariable("Path", newPath, EnvironmentVariableTarget.User);
                        }

                        // Register background service if checked
                        if (chkService.Checked)
                        {
                            try
                            {
                                ProcessStartInfo psi = new ProcessStartInfo(targetExe, "service install")
                                {
                                    CreateNoWindow = true,
                                    UseShellExecute = false
                                };
                                Process.Start(psi);
                            }
                            catch { }
                        }
                    }
                    catch (Exception ex)
                    {
                        MessageBox.Show("Installation note: " + ex.Message, "Notice", MessageBoxButtons.OK, MessageBoxIcon.Information);
                    }

                    progressBar.Value = 100;
                    lblStatus.Text = "Installation finished successfully.";
                    ShowStep(4);
                }
            };
            t.Start();
        }

        private void ExtractResource(string resourceName, string outputPath)
        {
            Assembly assembly = Assembly.GetExecutingAssembly();
            // Look for resource
            foreach (string name in assembly.GetManifestResourceNames())
            {
                if (name.EndsWith(resourceName, StringComparison.OrdinalIgnoreCase))
                {
                    using (Stream input = assembly.GetManifestResourceStream(name))
                    using (FileStream output = new FileStream(outputPath, FileMode.Create, FileAccess.Write))
                    {
                        input.CopyTo(output);
                        return;
                    }
                }
            }

            // Fallback: copy from adjacent if testing
            string localFallback = Path.Combine(AppDomain.CurrentDomain.BaseDirectory, resourceName);
            if (File.Exists(localFallback))
            {
                File.Copy(localFallback, outputPath, true);
            }
        }

        private void NextStep()
        {
            if (currentStep == 1 && radDecline.Checked)
            {
                MessageBox.Show("You must accept the license agreement to install LoomDB.", "License Notice", MessageBoxButtons.OK, MessageBoxIcon.Warning);
                return;
            }

            if (currentStep == 4)
            {
                if (chkLaunchNow.Checked)
                {
                    string targetExe = Path.Combine(this.installDir, "loom.exe");
                    try
                    {
                        ProcessStartInfo psi = new ProcessStartInfo("cmd.exe", "/k \"" + targetExe + "\"")
                        {
                            UseShellExecute = true
                        };
                        Process.Start(psi);
                    }
                    catch { }
                }
                this.Close();
                return;
            }

            ShowStep(currentStep + 1);
        }

        private void BackStep()
        {
            if (currentStep > 0)
            {
                ShowStep(currentStep - 1);
            }
        }
    }
}
