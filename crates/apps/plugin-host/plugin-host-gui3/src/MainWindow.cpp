//
// Created by Pedro Tacla Yamada on 10/11/21.
//

// You may need to build the project (run Qt uic code generator) to get "ui_MainWindow.h" resolved

#include "MainWindow.h"
#include "ui_MainWindow.h"

MainWindow::MainWindow (QWidget* parent) : QMainWindow (parent), ui (new Ui::MainWindow)
{
    ui->setupUi (this);

    leftDockWidget = new QDockWidget (this);
    leftDockWidget->setAllowedAreas (Qt::LeftDockWidgetArea);
    addDockWidget (Qt::LeftDockWidgetArea, leftDockWidget);
    leftDockWidget->setStyleSheet ("QDockWidget { min-width: 200px; }");

    contentsWindow = new QWidget (this);
    contentsWindow->setAutoFillBackground (true);
    button = new QPushButton ("Click me", contentsWindow);
    button->setStyleSheet("QPushButton::pressed { background-color: blue; }");

    setCentralWidget (contentsWindow);

    setStyleSheet ("QMainWindow { min-width: 500px; min-height: 500px; }"
                   "QMainWindow::separator { background: rgb(180, 180, 180); width: 1px; height: 1px; }");
    resize (800, 500);
}

MainWindow::~MainWindow ()
{
    delete ui;
}
