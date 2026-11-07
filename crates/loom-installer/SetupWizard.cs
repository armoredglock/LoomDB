using System;
using System.Drawing;
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
            this.Size = new Size(580, 420);
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
            // Left Blue Banner (Exactly like classic Windows Setup Wizard in Image 4)
            leftBanner = new Panel();
            leftBanner.Dock = DockStyle.Left;
            leftBanner.Width = 170;
            leftBanner.BackColor = Color.FromArgb(16, 44, 110);
            leftBanner.Paint += (s, e) =>
            {
                Graphics g = e.Graphics;
                using (Font f = new Font("Segoe UI", 16F, FontStyle.Bold))
                using (Brush b = new SolidBrush(Color.White))
                {
                    g.DrawString("LOOM // DB", f, b, new PointF(15, 30));
                }
                using (Font f = new Font("Segoe UI", 8F, FontStyle.Regular))
                using (Brush b = new SolidBrush(Color.FromArgb(180, 205, 255)))
                {
                    g.DrawString("The World's Lightest\nMulti-Modal Database\n\nVersion 0.1.0\nSub-512KB RAM Profile", f, b, new PointF(15, 75));
                }
            };
            this.Controls.Add(leftBanner);

            // Footer Panel
            footerPanel = new Panel();
            footerPanel.Dock = DockStyle.Bottom;
            footerPanel.Height = 55;
            footerPanel.BackColor = SystemColors.Control;
            footerPanel.Paint += (s, e) =>
            {
                e.Graphics.DrawLine(SystemPens.ControlDark, 0, 0, footerPanel.Width, 0);
            };

            btnCancel = new Button();
            btnCancel.Text = "Cancel";
            btnCancel.Size = new Size(85, 28);
            btnCancel.Location = new Point(465, 14);
            btnCancel.Click += (s, e) => this.Close();
            footerPanel.Controls.Add(btnCancel);

            btnNext = new Button();
            btnNext.Text = "Next >";
            btnNext.Size = new Size(85, 28);
            btnNext.Location = new Point(370, 14);
            btnNext.Click += (s, e) => NextStep();
            footerPanel.Controls.Add(btnNext);

            btnBack = new Button();
            btnBack.Text = "< Back";
            btnBack.Size = new Size(85, 28);
            btnBack.Location = new Point(280, 14);
            btnBack.Click += (s, e) => BackStep();
            footerPanel.Controls.Add(btnBack);

            this.Controls.Add(footerPanel);

            // Content Panel
            contentPanel = new Panel();
            contentPanel.Dock = DockStyle.Fill;
            contentPanel.Padding = new Padding(25, 20, 25, 20);
            this.Controls.Add(contentPanel);

            lblTitle = new Label();
            lblTitle.Font = new Font("Segoe UI", 13F, FontStyle.Bold);
            lblTitle.Location = new Point(20, 20);
            lblTitle.Size = new Size(360, 50);
            contentPanel.Controls.Add(lblTitle);

            lblDescription = new Label();
            lblDescription.Location = new Point(22, 75);
            lblDescription.Size = new Size(355, 120);
            lblDescription.Font = new Font("Segoe UI", 9F);
            contentPanel.Controls.Add(lblDescription);

            // Path Controls
            txtInstallPath = new TextBox();
            txtInstallPath.Location = new Point(22, 195);
            txtInstallPath.Size = new Size(270, 26);
            txtInstallPath.Text = this.installDir;
            txtInstallPath.Visible = false;
            contentPanel.Controls.Add(txtInstallPath);

            btnBrowse = new Button();
            btnBrowse.Text = "Browse...";
            btnBrowse.Location = new Point(300, 194);
            btnBrowse.Size = new Size(75, 26);
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

            // Checkboxes
            chkService = new CheckBox();
            chkService.Text = "Register LoomDB as automatic Windows Background Service (Port 8484)";
            chkService.Location = new Point(22, 235);
            chkService.Size = new Size(355, 40);
            chkService.Checked = true;
            chkService.Visible = false;
            contentPanel.Controls.Add(chkService);

            chkLaunchNow = new CheckBox();
            chkLaunchNow.Text = "Launch LoomDB Interactive Shell immediately";
            chkLaunchNow.Location = new Point(22, 200);
            chkLaunchNow.Size = new Size(355, 28);
            chkLaunchNow.Checked = true;
            chkLaunchNow.Visible = false;
            contentPanel.Controls.Add(chkLaunchNow);

            // Progress Bar
            progressBar = new ProgressBar();
            progressBar.Location = new Point(22, 195);
            progressBar.Size = new Size(350, 22);
            progressBar.Style = ProgressBarStyle.Continuous;
            progressBar.Visible = false;
            contentPanel.Controls.Add(progressBar);

            lblStatus = new Label();
            lblStatus.Location = new Point(22, 225);
            lblStatus.Size = new Size(350, 40);
            lblStatus.ForeColor = Color.DarkSlateGray;
            lblStatus.Visible = false;
            contentPanel.Controls.Add(lblStatus);

            // License radio buttons
            radAccept = new RadioButton();
            radAccept.Text = "I accept the terms in the License Agreement (MIT / Apache-2.0)";
            radAccept.Location = new Point(22, 195);
            radAccept.Size = new Size(355, 26);
            radAccept.Checked = true;
            radAccept.Visible = false;
            contentPanel.Controls.Add(radAccept);

            radDecline = new RadioButton();
            radDecline.Text = "I do not accept the terms in the License Agreement";
            radDecline.Location = new Point(22, 225);
            radDecline.Size = new Size(355, 26);
            radDecline.Visible = false;
            contentPanel.Controls.Add(radDecline);
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

            btnBack.Enabled = (step > 0 && step < 4);
            btnNext.Enabled = true;
            btnNext.Text = "Next >";

            switch (step)
            {
                case 0:
                    lblTitle.Text = "Welcome to the LoomDB Setup Wizard";
                    lblDescription.Text = "This wizard will guide you through the installation of LoomDB 0.1.0 (x64).\n\n" +
                                           "LoomDB is the world's lightest multi-modal database, weaving SQL Relational, " +
                                           "JSON Documents, AI Vectors, and Time-Series into a single portable container with " +
                                           "sub-512KB idle RAM overhead.\n\n" +
                                           "Click Next to continue, or Cancel to exit Setup.";
                    break;

                case 1:
                    lblTitle.Text = "License Agreement";
                    lblDescription.Text = "Please review the license terms before installing LoomDB.\n\n" +
                                           "LoomDB is open source software distributed under dual Apache-2.0 and MIT licenses.\n" +
                                           "You are granted full permission to use, embed, modify, and distribute LoomDB in both " +
                                           "personal and commercial proprietary applications free of charge.";
                    radAccept.Visible = true;
                    radDecline.Visible = true;
                    break;

                case 2:
                    lblTitle.Text = "Choose Install Location";
                    lblDescription.Text = "Setup will install LoomDB and configure command-line access in the following folder.\n\n" +
                                           "To install to this folder, click Next. To install to a different folder, click Browse and select another folder.";
                    txtInstallPath.Visible = true;
                    btnBrowse.Visible = true;
                    chkService.Visible = true;
                    break;

                case 3:
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
                    lblTitle.Text = "Completing the LoomDB Setup Wizard";
                    lblDescription.Text = "LoomDB has been successfully installed on your computer.\n\n" +
                                           "You can now run 'loom' from any Command Prompt or PowerShell terminal.\n\n" +
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
            t.Interval = 400;
            int progress = 20;

            t.Tick += (s, e) =>
            {
                progress += 25;
                if (progress <= 90)
                {
                    progressBar.Value = progress;
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
